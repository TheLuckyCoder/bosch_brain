use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::http::GlobalState;

#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CarStates {
    #[default]
    Standby,
    Config,
    RemoteControlled,
    AutonomousControlled,
}

pub fn router(global_state: Arc<GlobalState>) -> Router {
    Router::new()
        .route("/all", get(get_all_states))
        .route("/current", get(get_current_state))
        .route("/current/:new_state", post(set_current_state))
        .with_state(global_state)
}

async fn get_all_states() -> impl IntoResponse {
    Json(&[
        CarStates::Standby,
        CarStates::Config,
        CarStates::RemoteControlled,
        CarStates::AutonomousControlled,
    ])
}

async fn get_current_state(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    Json(*state.car_state.lock().await)
}

async fn set_current_state(
    State(state): State<Arc<GlobalState>>,
    Path(new_car_state): Path<CarStates>,
) -> StatusCode {
    let mut car_state_guard = state.car_state.lock().await;

    *car_state_guard = new_car_state;

    let mut sensor_manager = state.sensor_manager.lock().await;

    match new_car_state {
        CarStates::Standby => sensor_manager.reset(),
        CarStates::Config => sensor_manager.reset(),
        CarStates::RemoteControlled => {
            sensor_manager.listen_to_all_sensors();
        } // TODO Do something with it
        CarStates::AutonomousControlled => {
            sensor_manager.listen_to_all_sensors();
        }
    }

    StatusCode::OK
}
