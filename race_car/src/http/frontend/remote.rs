use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::sync::Arc;
use std::time::{Duration, Instant};

use askama::Template;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::{get, put};
use axum::{Form, Router};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use strum::IntoEnumIterator;
use tokio::task::yield_now;
use tokio::time::sleep;
use tracing::{error, info};
use v4l::buffer::Type;
use v4l::io::traits::{CaptureStream, Stream};
use v4l::prelude::UserptrStream;
use v4l::video::Capture;
use v4l::{Device, FourCC};

use sensors::name::SensorName;

use crate::actuators::ActuatorName;
use crate::http::config::JoystickConfig;
use crate::http::GlobalState;

const VIDEO_WIDTH: usize = 640;
const VIDEO_HEIGHT: usize = 480;

pub fn remote_router() -> Router<Arc<GlobalState>> {
    Router::new()
        .route("/", get(get_remote))
        .route("/joystick", put(update_joystick))
        .route("/ws", get(remote_ws))
        .route("/video", get(remote_video))
}

#[derive(Template)]
#[template(path = "pages/remote.html")]
struct RemoteTemplate {
    sensors: Vec<&'static str>,
    actuators: Vec<&'static str>,
    joystick: JoystickConfig,
    video_size: (usize, usize),
}

async fn get_remote(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    let sensor_manager = state.sensor_manager.lock().await;

    let sensors: Vec<_> = SensorName::iter()
        .filter(|sensor_name| sensor_manager.get_sensor(sensor_name).is_some())
        .map(|sensor_name| sensor_name.into())
        .collect();

    let actuators = ActuatorName::iter()
        .filter(|actuator_name| {
            state
                .actuator_manager
                .get_actuator_ref(*actuator_name)
                .is_some()
        })
        .map(|actuator_name| actuator_name.into())
        .collect();

    RemoteTemplate {
        sensors,
        actuators,
        joystick: state.server_config.lock().await.joystick.clone(),
        video_size: (VIDEO_WIDTH, VIDEO_HEIGHT),
    }
}

#[derive(Template)]
#[template(path = "components/joystick.html")]
struct JoystickTemplate {
    joystick: JoystickConfig,
}

#[serde_as]
#[derive(Default, Deserialize)]
struct JoystickQuery {
    #[serde_as(as = "Option<DisplayFromStr>")]
    size: Option<u8>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    opacity: Option<u8>,
}

async fn update_joystick(
    State(state): State<Arc<GlobalState>>,
    Form(query): Form<JoystickQuery>,
) -> impl IntoResponse {
    let mut server_config = state.server_config.lock().await;
    let default = JoystickConfig::default();

    let new_config = JoystickConfig {
        size: query.size.unwrap_or(default.size),
        opacity: query.opacity.unwrap_or(default.opacity),
    };
    server_config.joystick = new_config;
    server_config.save_to_file().unwrap();

    JoystickTemplate {
        joystick: new_config,
    }
}

async fn remote_ws(
    State(state): State<Arc<GlobalState>>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    info!("{addr} connected.");
    ws.on_upgrade(move |socket| handle_joystick_socket(socket, addr, state))
}

async fn handle_joystick_socket(
    mut socket: WebSocket,
    who: SocketAddr,
    global_state: Arc<GlobalState>,
) {
    let actuator_manager = global_state.actuator_manager.clone();

    loop {
        while let Some(message) = socket.recv().await {
            let Ok(message) = message else {
                println!("client {who} abruptly disconnected");
                return;
            };

            let message: WsMessage = match process_joystick_message(message, who) {
                ControlFlow::Continue(message) => {
                    if let Some(message) = message {
                        message
                    } else {
                        continue;
                    }
                }
                ControlFlow::Break(_) => return,
            };

            if let Some(motor) = actuator_manager.get_actuator(message.motors.x) {
                motor.lock().unwrap().set_value(message.joystick.x)
            }
            if let Some(motor) = actuator_manager.get_actuator(message.motors.y) {
                motor.lock().unwrap().set_value(message.joystick.y)
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

fn process_joystick_message(msg: Message, who: SocketAddr) -> ControlFlow<(), Option<WsMessage>> {
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

async fn remote_video(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    info!("`{addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_video_socket(socket, addr))
}

async fn handle_video_socket(mut socket: WebSocket, who: SocketAddr) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Create a new capture device with a few extra parameters
    let dev = Device::new(0).expect("Failed to open device");

    // Let's say we want to explicitly request another format
    let mut fmt = dev.format().expect("Failed to read format");
    fmt.width = VIDEO_WIDTH as u32;
    fmt.height = VIDEO_HEIGHT as u32;
    fmt.fourcc = FourCC::new(b"MJPG");
    // fmt.field_order = FieldOrder::Interlaced;

    let fmt = dev.set_format(&fmt).expect("Failed to write format");
    let mut stream = UserptrStream::with_buffers(&dev, Type::VideoCapture, 2)
        .expect("Failed to create buffer stream");
    stream.start().unwrap();
    println!("Format in use:\n{}", fmt);

    let mut send_task = tokio::spawn(async move {
        loop {
            let capture_instant = Instant::now();
            let (buf, meta) = stream.next().unwrap();
            let capture_ms = capture_instant.elapsed().as_millis();

            let transmission_instant = Instant::now();
            if ws_sender.send(Message::Binary(buf.to_vec())).await.is_err() {
                println!("client {who} abruptly disconnected");
                return;
            }

            // let transmission_ms = transmission_instant.elapsed().as_millis();
            // println!(
            //     "size: {}KB capture: {}ms, transmission: {}ms",
            //     meta.bytesused / 1024,
            //     capture_ms,
            //     transmission_ms
            // );
            yield_now().await;
        }
    });

    let mut recv_task = tokio::spawn(async move {
        loop {
            while let Some(message) = ws_receiver.next().await {
                let Ok(message) = message else {
                    println!("client {who} disconnected");
                    return;
                };

                if let Message::Close(_) = message {
                    break;
                }
            }

            sleep(Duration::from_millis(20)).await;
        }
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        _rv_a = &mut send_task => recv_task.abort(),
        _rv_b = &mut recv_task => send_task.abort(),
    }
}
