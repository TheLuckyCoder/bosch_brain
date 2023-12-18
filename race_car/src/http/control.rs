use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::post;
use axum::Router;

use shared::math::pid::PidController;

use crate::http::GlobalState;
use crate::sensors::Motor;

pub struct PidManager {
    pub steering: PidController,
}

pub fn router(state: Arc<GlobalState>) -> Router {
    Router::new()
        .route("/steering_pid", post(steering_pid))
        .with_state(state)
}

async fn steering_pid(State(state): State<Arc<GlobalState>>, Path(value): Path<f64>) {
    let mut pids = state.pids.blocking_lock();
    let mut motor = state.motor_driver.blocking_lock();

    let motor_value = pids.steering.compute(value);
    motor.set_motor_value(Motor::Steering, motor_value);
}
