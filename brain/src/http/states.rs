use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Json, Router};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use crate::http::GlobalState;
use serde::{Serialize, Deserialize};

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

    StatusCode::OK
}
