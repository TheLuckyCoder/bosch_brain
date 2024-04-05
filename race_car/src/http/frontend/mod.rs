use crate::http::GlobalState;
use crate::sensors::{AmbienceData, GpsCoordinates, ImuData, SensorData, SensorName, TimedSensorData};
use askama::Template;
use axum::extract::{ConnectInfo, Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::{get, put};
use axum::{Form, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::sync::Arc;
use axum::extract::ws::{Message, WebSocket};
use axum_extra::{headers, TypedHeader};
use multiqueue2::BroadcastReceiver;
use strum::IntoEnumIterator;
use tokio::sync::Mutex;
use tracing::info;
use crate::http::states::CarStates;
use crate::sensors::manager::SensorManager;

pub fn router(global_state: Arc<GlobalState>) -> Router {
    Router::new()
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .route("/", get(get_home))
        .route("/sensors", get(get_sensors))
        .route("/sensor_data", get(sensor_data_ws_handler))
        .route("/active_sensors", get(update_active_sensors))
        .with_state(global_state)
}

#[derive(Serialize)]
struct HomeSensor {
    name: &'static str,
    active: bool,
    icon_path: String,
}

#[derive(Template)]
#[template(path = "pages/home.html")]
struct HomeTemplate {
    state: &'static str,
    sensors: Vec<HomeSensor>,
}

async fn get_home(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    let car_state = *state.car_state.lock().await;
    let sensor_manager = state.sensor_manager.lock().await;

    let sensors: Vec<_> = SensorName::iter()
        .map(|sensor_name| {
            let active = sensor_manager.get_sensor(&sensor_name).is_some();
            HomeSensor {
                name: sensor_name.into(),
                active,
                icon_path: format!(
                    "assets/icons/{}_{}.png",
                    sensor_name.to_string().to_lowercase(),
                    if active { "active" } else { "inactive" }
                ),
            }
        })
        .collect();

    HomeTemplate {
        state: car_state.into(),
        sensors,
    }
}

#[derive(Serialize)]
struct Sensor {
    name: &'static str,
    enabled: bool,
    icon_path: String,
}

#[derive(Template)]
#[template(path = "pages/sensors.html")]
struct SensorTemplate {
    state: &'static str,
    sensors: Vec<Sensor>,
}

async fn get_sensors(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    let car_state = *state.car_state.lock().await;
    let sensor_manager = state.sensor_manager.lock().await;
    let udp_broadcast = state.udp_manager.lock().await.get_active_sensors().clone();

    let sensors: Vec<_> = SensorName::iter()
        .filter(|sensor_name| sensor_manager.get_sensor(sensor_name).is_some())
        .map(|sensor_name| {
            let active = sensor_manager.get_sensor(&sensor_name).is_some();
            let enabled = udp_broadcast.contains(&sensor_name);
            Sensor {
                name: sensor_name.into(),
                enabled,
                icon_path: format!(
                    "assets/icons/{}_{}.png",
                    sensor_name.to_string().to_lowercase(),
                    if active { "active" } else { "inactive" }
                ),
            }
        })
        .collect();

    SensorTemplate {
        state: car_state.into(),
        sensors,
    }
}

async fn update_active_sensors(
    State(state): State<Arc<GlobalState>>,
    Query(active_sensors): Query<HashMap<SensorName, String>>,
) -> impl IntoResponse {
    info!("New Active Sensors: {:?}", active_sensors);

    let active_sensors: Vec<_> = active_sensors.into_iter()
        .filter(|(_, value)| value == "on")
        .map(|(sensor_name, _)| sensor_name)
        .collect();

    // let mut udp_manager = state.udp_manager.lock().await;
    // udp_manager.set_active_sensors(active_sensors, format!("{}:3001", addr.ip()));
    *state.active_sensors.lock().await = active_sensors;

    "Waiting for Sensor Data"
}

async fn sensor_data_ws_handler(
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
    println!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

#[derive(Template)]
#[template(path = "components/ws_sensor_data.html")]
struct WsMessageResponseTemplate {
    message: String,
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(mut socket: WebSocket, who: SocketAddr, global_state: Arc<GlobalState>) {
    let sensor_manager = &global_state.sensor_manager;
    sensor_manager.lock().await.start_listening_to_sensors();
    //send a ping (unsupported by some browsers) just to kick things off and get a response
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        println!("Pinged {who}...");
    } else {
        println!("Could not send ping {who}!");
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    // Since each client gets individual statemachine, we can pause handling
    // when necessary to wait for some external event (in this case illustrated by sleeping).
    // Waiting for this client to finish getting its greetings does not prevent other clients from
    // connecting to server and receiving their greetings.
    let receiver = sensor_manager.lock().await.get_data_receiver().add_stream();
    loop {
        let active_sensors = global_state.active_sensors.lock().await.clone();
        let data = reader_mode(&active_sensors, &receiver);

        println!("Active: {active_sensors:?}; Sending data: {data:?}");
        if let Some(message) = data {
            if socket
                .send(Message::Text(WsMessageResponseTemplate { message }.to_string()))
                .await
                .is_err()
            {
                println!("client {who} abruptly disconnected");
                return;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
    }
}
#[derive(Default, serde::Serialize)]
struct UdpData {
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    imu: Option<ImuData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ultrasonic: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gps: Option<GpsCoordinates>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ambience: Option<AmbienceData>,
}

impl UdpData {
    /// Checks if the struct contains any data
    fn is_empty(&self) -> bool {
        self.imu.is_none()
            && self.ultrasonic.is_none()
            && self.gps.is_none()
            && self.ambience.is_none()
    }
}

fn reader_mode(active_sensors: &[SensorName], receiver: &BroadcastReceiver<TimedSensorData>) -> Option<String> {
    let mut udp_data = receiver
        .try_iter()
        .fold(UdpData::default(), |mut udp, sensor_data| {
            match sensor_data.data {
                SensorData::Imu(imu) => udp.imu = Some(imu),
                SensorData::Distance(distance) => udp.ultrasonic = Some(distance),
                SensorData::Gps(gps) => udp.gps = Some(gps),
                SensorData::Ambience(ambience) => udp.ambience = Some(ambience),
                _ => {}
            }
            udp
        });

    if !active_sensors.contains(&SensorName::Imu) {
        udp_data.imu = None;
    }

    if !active_sensors.contains(&SensorName::Ultrasonic) {
        udp_data.ultrasonic = None;
    }

    if !active_sensors.contains(&SensorName::Gps) {
        udp_data.gps = None;
    }

    if !active_sensors.contains(&SensorName::Ambience) {
        udp_data.ambience = None;
    }

    if !udp_data.is_empty() {
        Some(serde_json::to_string(&udp_data).expect("Failed to serialize UDP data"))
    } else {
        None
    }
}

/// helper to print contents of messages to stdout. Has special treatment for Close.
fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            println!(">>> {who} sent str: {t:?}");
        }
        Message::Binary(d) => {
            println!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                println!(">>> {who} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            println!(">>> {who} sent pong with {v:?}");
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            println!(">>> {who} sent ping with {v:?}");
        }
    }
    ControlFlow::Continue(())
}
