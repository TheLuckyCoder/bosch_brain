use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::sync::Arc;

use crate::actuators::ActuatorName;
use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, Path, State, WebSocketUpgrade};
use axum::routing::{get, put};
use axum::Router;
use axum_extra::{headers, TypedHeader};
use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use strum::IntoEnumIterator;
use tracing::{error, info};

use crate::http::GlobalState;

pub fn actuators_router(global_state: Arc<GlobalState>) -> Router {
    Router::new()
        .route("/actuators", get(get_actuators))
        .route("/actuators/ws", get(actuators_ws))
        .route("/pause_actuator/:name", put(pause_actuator))
        .route("/resume_actuator/:name", put(resume_actuator))
        .route("/stop_actuator/:name", put(stop_actuator))
        .with_state(global_state)
}

struct ActuatorTemplateContent {
    name: &'static str,
    active: bool,
    paused: bool,
    configuration: Option<String>,
}

#[derive(Template)]
#[template(path = "pages/actuators.html")]
struct ActuatorsTemplate {
    actuators: Vec<ActuatorTemplateContent>,
}

async fn get_actuators(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    let actuators = ActuatorName::iter()
        .map(
            |actuator_name| match state.actuator_manager.get_actuator_ref(actuator_name) {
                None => ActuatorTemplateContent {
                    name: actuator_name.into(),
                    active: false,
                    paused: false,
                    configuration: None,
                },
                Some(actuator) => {
                    let guard = actuator.lock().unwrap();

                    ActuatorTemplateContent {
                        name: actuator_name.into(),
                        active: true,
                        paused: guard.is_paused(),
                        configuration: guard.get_config_html(),
                    }
                }
            },
        )
        .collect();

    ActuatorsTemplate { actuators }
}

#[derive(Template)]
#[template(path = "responses/pause_resume_actuators_response.html")]
struct PauseResumeResponse {
    actuator: ActuatorTemplateContent,
}

async fn pause_actuator(
    State(state): State<Arc<GlobalState>>,
    Path(name): Path<ActuatorName>,
) -> impl IntoResponse {
    let actuator = state.actuator_manager.get_actuator_ref(name);

    let mut actuator = actuator.unwrap().lock().unwrap();
    actuator.pause();

    PauseResumeResponse {
        actuator: ActuatorTemplateContent {
            name: actuator.name().into(),
            active: true,
            paused: actuator.is_paused(),
            configuration: actuator.get_config_html(),
        },
    }
}

async fn resume_actuator(
    State(state): State<Arc<GlobalState>>,
    Path(name): Path<ActuatorName>,
) -> impl IntoResponse {
    let actuator = state.actuator_manager.get_actuator_ref(name);

    let mut actuator = actuator.unwrap().lock().unwrap();
    actuator.resume();

    PauseResumeResponse {
        actuator: ActuatorTemplateContent {
            name: actuator.name().into(),
            active: true,
            paused: actuator.is_paused(),
            configuration: actuator.get_config_html(),
        },
    }
}

async fn stop_actuator(
    State(state): State<Arc<GlobalState>>,
    Path(name): Path<ActuatorName>,
) -> impl IntoResponse {
    let actuator = state.actuator_manager.get_actuator_ref(name);

    let mut actuator = actuator.unwrap().lock().unwrap();
    actuator.stop();

    PauseResumeResponse {
        actuator: ActuatorTemplateContent {
            name: actuator.name().into(),
            active: true,
            paused: actuator.is_paused(),
            configuration: actuator.get_config_html(),
        },
    }
}

async fn actuators_ws(
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

            match process_message(message, who) {
                ControlFlow::Continue(Some(motor_values)) => {
                    let Some(actuator) = actuator_manager.get_actuator_ref(motor_values.name)
                    else {
                        continue;
                    };

                    actuator.lock().unwrap().set_value(motor_values.value);

                    println!("{:?}", motor_values.value)
                }
                ControlFlow::Continue(None) => continue,
                ControlFlow::Break(_) => return,
            }
        }
    }
}

#[serde_as]
#[derive(Deserialize)]
struct WsMessage {
    name: ActuatorName,
    #[serde_as(as = "DisplayFromStr")]
    value: f64,
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
