use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::sync::Arc;

use askama::Template;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use axum_extra::{headers, TypedHeader};
use serde::Deserialize;
use strum::IntoEnumIterator;
use tracing::{error, info};

use crate::actuator::ActuatorName;
use crate::http::GlobalState;
use crate::sensors::SensorName;

pub fn remote_router(global_state: Arc<GlobalState>) -> Router {
    Router::new()
        .route("/remote", get(get_remote))
        .route("/remote/ws", get(remote_ws))
        .with_state(global_state)
}

#[derive(Template)]
#[template(path = "pages/remote.html")]
struct RemoteTemplate {
    sensors: Vec<&'static str>,
    actuators: Vec<&'static str>,
}

async fn get_remote(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    let sensor_manager = state.sensor_manager.lock().await;

    let sensors: Vec<_> = SensorName::iter()
        .filter(|sensor_name| sensor_manager.get_sensor(sensor_name).is_some())
        .map(|sensor_name| sensor_name.into())
        .collect();

    let actuators = ActuatorName::iter()
        .filter(|actuator_name| state.actuator_manager.get_actuator_ref(actuator_name.clone()).is_some())
        .map(|actuator_name| actuator_name.into())
        .collect();

    RemoteTemplate { sensors, actuators }
}

pub async fn remote_ws(
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

async fn handle_socket(mut socket: WebSocket, who: SocketAddr, global_state: Arc<GlobalState>) {
    let actuator_manager = global_state.actuator_manager.clone();

    loop {
        while let Some(message) = socket.recv().await {
            let Ok(message) = message else {
                println!("client {who} abruptly disconnected");
                return;
            };

            let message: WsMessage = match process_message(message, who) {
                ControlFlow::Continue(message) => {
                    if let Some(message) = message {
                        message
                    } else {
                        continue
                    }
                }
                ControlFlow::Break(_) => return,
            };


            if let Some(motor) = actuator_manager.get_actuator(message.motors.x)
            {
                motor.lock().unwrap().set_value(message.joystick.y)
            }
            if let Some(motor) = actuator_manager.get_actuator(message.motors.y)
            {
                motor.lock().unwrap().set_value(message.joystick.x)
            }
        }
    }
}

#[derive(Deserialize)]
struct WsMotors {
    x: ActuatorName,
    y: ActuatorName,
}

#[derive(Deserialize)]
struct WsJoystick {
    x: f64,
    y: f64,
}

#[derive(Deserialize)]
struct WsMessage {
    motors: WsMotors,
    joystick: WsJoystick,
}

fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), Option<WsMessage>> {
    match msg {
        Message::Text(text) => {
            info!(">>> {who} sent str: {text:?}");
            let Ok(values) = serde_json::from_str::<WsMessage>(&text) else {
                error!("Failed to parse message");
                return ControlFlow::Continue(None);
            };

            ControlFlow::Continue(Some(values))
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
            ControlFlow::Break(())
        }
        _ => ControlFlow::Continue(None),
    }
}
