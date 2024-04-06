use crate::http::GlobalState;
use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum_extra::{headers, TypedHeader};
use futures_util::{SinkExt, StreamExt};
use multiqueue2::BroadcastReceiver;
use serde::de::IgnoredAny;
use serde::{Deserialize};
use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::str::FromStr;
use std::sync::Arc;
use strum::IntoEnumIterator;
use tokio::sync::Mutex;
use tracing::{error, info};
use crate::sensors::{SensorData, SensorName, TimedSensorData};

#[derive(Template)]
#[template(path = "pages/sensors.html")]
struct SensorTemplate {
    state: &'static str,
    sensors: Vec<&'static str>,
}

pub async fn get_sensors(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    let car_state = *state.car_state.lock().await;
    let sensor_manager = state.sensor_manager.lock().await;

    let sensors: Vec<_> = SensorName::iter()
        .filter(|sensor_name| sensor_manager.get_sensor(sensor_name).is_some())
        .map(|sensor_name| sensor_name.into())
        .collect();

    SensorTemplate {
        state: car_state.into(),
        sensors,
    }
}

pub async fn sensor_data_ws_handler(
    State(state): State<Arc<GlobalState>>,
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    info!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

#[derive(Template)]
#[template(path = "components/ws_sensor_data.html")]
struct WsMessageResponseTemplate {
    sensors_data: Vec<SensorData>,
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(socket: WebSocket, who: SocketAddr, global_state: Arc<GlobalState>) {
    let sensor_manager = &global_state.sensor_manager;
    sensor_manager.lock().await.start_listening_to_sensors();

    let active_sensors: Arc<Mutex<Vec<SensorName>>> = Default::default();
    let receiver = sensor_manager.lock().await.get_data_receiver().add_stream();

    let (mut ws_sender, mut ws_receiver) = socket.split();

    let active_sensors_clone = active_sensors.clone();
    let mut send_task = tokio::spawn(async move {
        let active_sensors = active_sensors_clone;

        loop {
            let collected_sensors = {
                let active_sensors_value = active_sensors.lock().await;
                reader_mode(&active_sensors_value, &receiver)
            };

            let sensors_data: Vec<_> = collected_sensors.into_values().collect();

            println!("Sending");
            if !sensors_data.is_empty()
                && ws_sender
                    .send(Message::Text(
                        WsMessageResponseTemplate { sensors_data }.render().unwrap(),
                    ))
                    .await
                    .is_err()
            {
                println!("client {who} abruptly disconnected");
                return;
            }

            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        }
    });

    // This second task will receive messages from client and print them on server console
    let mut recv_task = tokio::spawn(async move {
        loop {
            while let Some(message) = ws_receiver.next().await {
                if let Ok(message) = message {
                    match process_message(message, who) {
                        ControlFlow::Continue(new_active_sensors) => {
                            if let Some(new_active_sensors) = new_active_sensors {
                                *active_sensors.lock().await = new_active_sensors;
                            }
                        }
                        ControlFlow::Break(_) => return,
                    }
                } else {
                    println!("client {who} abruptly disconnected");
                    return;
                }
            }

            println!("Finished processing messages");
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        _rv_a = &mut send_task => recv_task.abort(),
        _rv_b = &mut recv_task => send_task.abort(),
    }

    // returning from the handler closes the websocket connection
    println!("Websocket context {who} destroyed");
}

fn reader_mode(
    active_sensors: &[SensorName],
    receiver: &BroadcastReceiver<TimedSensorData>,
) -> BTreeMap<SensorName, SensorData> {
    receiver
        .try_iter()
        .map(|sensor_data| (sensor_data.data.get_sensor_name(), sensor_data.data))
        .filter(|(sensor_name, _)| active_sensors.contains(sensor_name))
        .collect()
}

#[derive(Deserialize)]
struct WsMessage {
    #[serde(rename = "HEADERS")]
    _headers: IgnoredAny,
    #[serde(flatten)]
    sensors: HashMap<String, String>,
}

/// helper to print contents of messages to stdout. Has special treatment for Close.
fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), Option<Vec<SensorName>>> {
    match msg {
        Message::Text(t) => {
            info!(">>> {who} sent str: {t:?}");
            let Ok(values) = serde_json::from_str::<WsMessage>(&t) else {
                error!("Failed to parse message");
                return ControlFlow::Continue(None);
            };
            
            let active_sensors: Vec<_> = values
                .sensors
                .into_iter()
                .filter(|(_, value)| value == "on")
                .filter_map(|(sensor_name, _)| SensorName::from_str(&sensor_name).ok())
                .collect();

            return ControlFlow::Continue(Some(active_sensors));
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                info!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                info!(">>> {who} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }
        // Message::Pong(v) => info!(">>> {who} sent pong with {v:?}"),
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        // Message::Ping(v) => info!(">>> {who} sent ping with {v:?}"),
        _ => {}
    }
    ControlFlow::Continue(None)
}
