//! HTTP routes for manually controlling the car's motors.
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum::http::StatusCode;
use strum::IntoEnumIterator;
use tokio::task;
use tracing::info;

use crate::actuators::ActuatorName;
use crate::http::GlobalState;

/// Creates an object that manages all the actuator routes
pub fn router() -> Router<Arc<GlobalState>> {
    Router::new()
        .route("/", get(get_actuators))
        .route("/active", get(get_active_actuators))
        .route("/stop/:actuator", post(stop_actuator))
        .route("/stop", post(stop_actuator))
        .route("/set/:actuator/:value", post(set_actuator_value))
        .route("/pause/:actuator", post(pause_actuator))
        .route("/pause", post(pause_actuator))
        .route("/is_paused/:actuator", get(is_paused))
        .route("/resume/:actuator", post(resume_actuator))
        .route("/resume", post(resume_actuator))
        .route("/params/:actuator", get(get_actuator_parameters))
        .route("/params/:actuator", post(set_actuator_parameters))
}

/// Returns a list of all actuators
async fn get_actuators() -> impl IntoResponse {
    Json(ActuatorName::iter().collect::<Vec<_>>())
}

async fn get_active_actuators(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    Json(state.actuator_manager.get_active_actuators().into_iter().map(|(name, _)| name).collect::<Vec<_>>())
}

/// Returns the current parameters for the given actuator
async fn get_actuator_parameters(
    State(state): State<Arc<GlobalState>>,
    Path(actuator_name): Path<ActuatorName>,
) -> impl IntoResponse {
    let actuator = state.actuator_manager.get_actuator_ref(actuator_name).unwrap().lock().unwrap();

    match actuator.get_config() {
        None => (StatusCode::NOT_IMPLEMENTED, "Not supported for this actuator".to_string()),
        Some(config) => (StatusCode::OK, config),
    }
}

/// Sets the parameters for the given actuator
async fn set_actuator_parameters(
    State(state): State<Arc<GlobalState>>,
    Path(actuator_name): Path<ActuatorName>,
    Json(params): Json<String>,
) {
    info!("Motor params received: {params}");

    task::spawn_blocking(move || {
        let mut actuator = state.actuator_manager.get_actuator_ref(actuator_name).unwrap().lock().unwrap();

        actuator.save_config(params)
    })
    .await
    .unwrap()
    .unwrap();
}

/// Stops the given actuator, or all motors if no actuator is specified
async fn stop_actuator(
    State(state): State<Arc<GlobalState>>,
    actuator: Option<Path<ActuatorName>>,
) {
    if let Some(Path(actuator)) = actuator {
        state
            .actuator_manager
            .get_actuator_ref(actuator)
            .unwrap()
            .lock()
            .unwrap()
            .stop();
    } else {
        for (_, actuator) in state.actuator_manager.get_active_actuators() {
            actuator.lock().unwrap().stop();
        }
    }
}

/// Sets the value for the given actuator
async fn set_actuator_value(
    State(state): State<Arc<GlobalState>>,
    Path((actuator, value)): Path<(ActuatorName, f64)>,
) {
    state
        .actuator_manager
        .get_actuator_ref(actuator)
        .unwrap()
        .lock()
        .unwrap()
        .set_value(value);
}

/// Pauses the given actuator, or all motors if no actuator is specified
async fn pause_actuator(
    State(state): State<Arc<GlobalState>>,
    actuator: Option<Path<ActuatorName>>,
) {
    if let Some(Path(actuator)) = actuator {
        state
            .actuator_manager
            .get_actuator_ref(actuator)
            .unwrap()
            .lock()
            .unwrap()
            .pause();
    } else {
        for (_, actuator) in state.actuator_manager.get_active_actuators() {
            actuator.lock().unwrap().pause();
        }
    }
}

async fn is_paused(
    State(state): State<Arc<GlobalState>>,
    Path(actuator): Path<ActuatorName>,
) -> impl IntoResponse {
        Json(state
            .actuator_manager
            .get_actuator_ref(actuator)
            .unwrap()
            .lock()
            .unwrap()
            .is_paused())
}

/// Resumes the given actuator, or all motors if no actuator is specified
async fn resume_actuator(
    State(state): State<Arc<GlobalState>>,
    actuator: Option<Path<ActuatorName>>,
) {
    if let Some(Path(actuator)) = actuator {
        state
            .actuator_manager
            .get_actuator_ref(actuator)
            .unwrap()
            .lock()
            .unwrap()
            .resume();
    } else {
        for (_, actuator) in state.actuator_manager.get_active_actuators() {
            actuator.lock().unwrap().resume();
        }
    }
}
