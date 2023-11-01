use std::sync::Arc;
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use sensors::{Motor, MotorParams};
use crate::http::GlobalState;
use crate::http::states::CarStates;

pub fn router(global_state: Arc<GlobalState>) -> Router {
    Router::new()
        .route("/all", get(get_motors))
        .route("/params/:motor", get(get_motor_parameters))
        .route("/params/:motor", post(set_motor_parameters))
        .route("/stop/:motor", post(stop_motor))
        .route("/start/:motor/:value", post(set_motor_value))
        .route("/remote_control", post(set_steering_and_acceleration))
        .with_state(global_state)
}

async fn get_motors() -> impl IntoResponse {
    Json([
        Motor::Steering,
        Motor::Acceleration
    ])
}

async fn get_motor_parameters(
    State(state): State<Arc<GlobalState>>,
    Path(motor): Path<Motor>,
) -> impl IntoResponse {
    let motor_driver = state.motor_driver.lock().await;

    Json(motor_driver.get_params(motor))
}

async fn set_motor_parameters(
    State(state): State<Arc<GlobalState>>,
    Path(motor): Path<Motor>,
    Json(params): Json<MotorParams>,
) {
    let mut motor_driver = state.motor_driver.lock().await;

    motor_driver.set_params(motor, params);
}

async fn stop_motor(
    State(state): State<Arc<GlobalState>>,
    Path(motor): Path<Motor>,
) {
    let mut motor_driver = state.motor_driver.lock().await;

    motor_driver.stop_motor(motor);
}

async fn set_motor_value(
    State(state): State<Arc<GlobalState>>,
    Path((motor, value)): Path<(Motor, f64)>,
) {
    let mut motor_driver = state.motor_driver.lock().await;

    motor_driver.set_motor_value(motor, value);
}

#[derive(serde::Deserialize)]
struct AccelerationAndSteering {
    pub acceleration: f64,
    pub steering: f64,
}

async fn set_steering_and_acceleration(
    State(state): State<Arc<GlobalState>>,
    Json(values): Json<AccelerationAndSteering>,
) -> StatusCode {
    if *state.car_state.lock().await != CarStates::RemoteControlled {
        return StatusCode::BAD_REQUEST;
    }

    let mut motor = state.motor_driver.lock().await;

    motor.set_motor_value(Motor::Acceleration, values.acceleration);
    motor.set_motor_value(Motor::Steering, values.steering);

    StatusCode::OK
}
