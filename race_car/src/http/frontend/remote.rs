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
use tokio::task::yield_now;
use tokio::time::sleep;
use tracing::{error, info};
use v4l::buffer::Type;
use v4l::io::traits::{CaptureStream, Stream};
use v4l::prelude::{MmapStream};
use v4l::video::Capture;
use v4l::{Device, FourCC};

use sensors::name::SensorName;

use crate::actuators::ActuatorName;
use crate::http::config::{JoystickConfig, ServerConfig, VideoConfig};
use crate::http::GlobalState;

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
    actuators: Vec<ActuatorName>,
    config: ServerConfig,
    main_actuator_name: ActuatorName,
    main_actuator_paused: bool,
}

async fn get_remote(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    let sensors_names = state
        .sensor_manager
        .lock()
        .await
        .get_active_sensors()
        .iter()
        .map(|(name, _)| name.into())
        .collect();

    let active_actuators = state.actuator_manager.get_active_actuators();
    let active_actuators_name: Vec<_> = active_actuators.iter().map(|(name, _)| *name).collect();

    let config = state.server_config.lock().await.clone();

    RemoteTemplate {
        sensors: sensors_names,
        actuators: active_actuators_name,
        main_actuator_name: config.joystick.x_axis,
        main_actuator_paused: active_actuators
            .iter()
            .find(|(name, actuator)| *name == config.joystick.x_axis)
            .map_or(true, |(_, actuator)| actuator.lock().unwrap().is_paused()),
        config,
    }
}

#[derive(Template)]
#[template(path = "components/joystick.html")]
struct JoystickTemplate {
    config: ServerConfig,
}

#[serde_as]
#[derive(Deserialize)]
struct JoystickQuery {
    #[serde_as(as = "DisplayFromStr")]
    size: u8,
    #[serde_as(as = "DisplayFromStr")]
    opacity: u8,
    x_axis: ActuatorName,
    y_axis: ActuatorName,
}

async fn update_joystick(
    State(state): State<Arc<GlobalState>>,
    Form(query): Form<JoystickQuery>,
) -> impl IntoResponse {
    let new_config = JoystickConfig {
        size: query.size,
        opacity: query.opacity,
        x_axis: query.x_axis,
        y_axis: query.y_axis,
    };

    let mut server_config = state.server_config.lock().await;
    server_config.joystick = new_config;
    server_config.save_to_file().unwrap();

    JoystickTemplate {
        config: server_config.clone(),
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

            let message: WsJoystick = match process_joystick_message(message, who) {
                ControlFlow::Continue(message) => {
                    if let Some(message) = message {
                        message
                    } else {
                        continue;
                    }
                }
                ControlFlow::Break(_) => return,
            };
            let joystick_config = global_state.server_config.lock().await.joystick;

            if let Some(motor) = actuator_manager.get_actuator(joystick_config.x_axis) {
                motor.lock().unwrap().set_value(message.x)
            }
            if let Some(motor) = actuator_manager.get_actuator(joystick_config.y_axis) {
                motor.lock().unwrap().set_value(message.y)
            }
        }
    }
}

#[derive(Deserialize)]
struct WsJoystick {
    x: f64,
    y: f64,
}

fn process_joystick_message(msg: Message, who: SocketAddr) -> ControlFlow<(), Option<WsJoystick>> {
    match msg {
        Message::Text(text) => {
            info!(">>> {who} sent str: {text:?}");
            let Ok(values) = serde_json::from_str::<WsJoystick>(&text) else {
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
    State(state): State<Arc<GlobalState>>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    info!("`{addr} connected.");
    let video_config = state.server_config.lock().await.video;
    ws.on_upgrade(move |socket| handle_video_socket(socket, addr, video_config))
}

async fn handle_video_socket(socket: WebSocket, who: SocketAddr, video_config: VideoConfig) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Create a new capture device with a few extra parameters
    let dev = Device::new(video_config.device_index as usize).expect("Failed to open device");

    // Let's say we want to explicitly request another format
    let mut fmt = dev.format().expect("Failed to read format");
    fmt.width = video_config.width as u32;
    fmt.height = video_config.height as u32;
    fmt.fourcc = FourCC::new(b"MJPG");

    if fmt.width == 0 || fmt.height == 0 {
        return;
    }

    let fmt = dev.set_format(&fmt).expect("Failed to write format");
    let mut stream = MmapStream::with_buffers(&dev, Type::VideoCapture, 1)
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
