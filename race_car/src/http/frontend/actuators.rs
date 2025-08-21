use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::sync::Arc;

use crate::actuators::ActuatorName;
use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, Path, State, WebSocketUpgrade};
use axum::routing::{get, post, put};
use axum::{Form, Router};
use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use strum::IntoEnumIterator;
use tracing::{error, info};

use crate::http::GlobalState;

pub fn actuators_router() -> Router<Arc<GlobalState>> {
    Router::new()
        .route("/", get(get_actuators))
        .route("/ws", get(actuators_ws))
        .route("/pause/:name", put(pause_actuator))
        .route("/resume/:name", put(resume_actuator))
        .route("/stop/:name", put(stop_actuator))
        .route("/inputs", post(update_inputs))
}

const VALUE_STEPS: [f32; 6] = [0.001, 0.005, 0.01, 0.05, 0.1, 0.5];

struct InputsParams {
    value: f64,
    step_input: u8,
    step_value: f32,
    min: f64,
    max: f64,
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
    inputs: InputsParams,
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

    ActuatorsTemplate {
        actuators,
        inputs: InputsParams {
            value: 0.0,
            step_input: 0,
            step_value: VALUE_STEPS[0],
            min: -1.0,
            max: 1.0,
        },
    }
}

#[derive(Template)]
#[template(path = "responses/pause_resume_actuators_response.html")]
struct PauseResumeResponse {
    actuator_name: &'static str,
    actuator_paused: bool,
}

async fn pause_actuator(
    State(state): State<Arc<GlobalState>>,
    Path(name): Path<ActuatorName>,
) -> impl IntoResponse {
    let actuator = state.actuator_manager.get_actuator_ref(name);

    let mut actuator = actuator.unwrap().lock().unwrap();
    actuator.pause();

    PauseResumeResponse {
        actuator_name: actuator.name().into(),
        actuator_paused: actuator.is_paused(),
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
        actuator_name: actuator.name().into(),
        actuator_paused: actuator.is_paused(),
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
        actuator_name: actuator.name().into(),
        actuator_paused: actuator.is_paused(),
    }
}

async fn actuators_ws(
    State(state): State<Arc<GlobalState>>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    info!("{addr} connected.");
    ws.on_upgrade(move |socket| handle_actuators_socket(socket, addr, state))
}

async fn handle_actuators_socket(
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

            match process_message(message, who) {
                ControlFlow::Continue(Some(motor_values)) => {
                    let Some(actuator) = actuator_manager.get_actuator_ref(motor_values.name)
                    else {
                        continue;
                    };

                    match actuator.lock() {
                        Ok(mut guard) => {
                            match motor_values.data {
                                WsData::Value(value) => guard.set_command(value),
                                WsData::Config(config) => if let Err(e) = guard.save_config(config) {
                                    error!("Failed saving config: {e}");
                                },
                            }
                        }
                        Err(poisoned) => {
                            // Log the underlying cause
                            error!("Mutex poisoned: {:?}", poisoned);
                        }
                    }

                }
                ControlFlow::Continue(None) => continue,
                ControlFlow::Break(_) => return,
            }
        }
    }
}

#[derive(Deserialize)]
struct WsMessage {
    name: ActuatorName,
    #[serde(flatten)]
    data: WsData,
}

#[serde_as]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
enum WsData {
    Value(#[serde_as(as = "DisplayFromStr")] f64),
    Config(String),
}

fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), Option<WsMessage>> {
    match msg {
        Message::Text(text) => {
            info!(">>> {who} sent str: {text:?}");
            let values = match serde_json::from_str::<WsMessage>(&text) {
                Ok(values) => values,
                Err(e) => {
                    error!("Failed to parse message: {e}");
                    return ControlFlow::Continue(None);
                }
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

#[serde_as]
#[derive(Deserialize)]
struct InputsQuery {
    name: String,
    #[serde_as(as = "DisplayFromStr")]
    value: f64,
    #[serde_as(as = "DisplayFromStr")]
    step: u8,
    #[serde_as(as = "DisplayFromStr")]
    min: f64,
    #[serde_as(as = "DisplayFromStr")]
    max: f64,
}

#[derive(Template)]
#[template(path = "components/actuators_inputs.html")]
struct InputsTemplate {
    actuator_name: String,
    inputs: InputsParams,
}

async fn update_inputs(Form(inputs): Form<InputsQuery>) -> impl IntoResponse {
    InputsTemplate {
        actuator_name: inputs.name,
        inputs: InputsParams {
            value: inputs.value.clamp(inputs.min, inputs.max),
            step_input: inputs.step,
            step_value: VALUE_STEPS[(inputs.step as usize).min(VALUE_STEPS.len())],
            min: inputs.min.min(inputs.max),
            max: inputs.max.max(inputs.min),
        },
    }
}
