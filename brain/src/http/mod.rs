use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use tokio::sync::Mutex;
use tokio::task;

use sensors::SensorManager;

use crate::motor_manager::MotorManager;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CarStates {
    Standby = 0,
    Config = 1,
    RemoteControlled = 2,
    AutonomousControlled = 3,
}

#[derive(Clone)]
pub struct GlobalState {
    pub car_state: Arc<Mutex<CarStates>>,
    pub sensor_manager: Arc<SensorManager>,
    pub motor_manager: Arc<MotorManager>,
}

async fn get_current_state(State(state): State<GlobalState>) -> Json<u8> {
    Json(*state.car_state.lock().await as u8)
}

async fn set_current_state(
    State(mut state): State<GlobalState>,
    Path(new_car_state): Path<u8>,
) -> StatusCode {
    let mut car_state_guard = state.car_state.lock().await;

    let car_state = match new_car_state {
        0 => CarStates::Standby,
        1 => CarStates::Config,
        2 => CarStates::RemoteControlled,
        3 => CarStates::AutonomousControlled,
        _ => return StatusCode::BAD_REQUEST,
    };

    *car_state_guard = car_state;

    StatusCode::OK
}

async fn get_all_available_sensors(
    State(state): State<GlobalState>,
) -> Json<HashMap<&'static str, bool>> {
    let sensor_manager = &state.sensor_manager;

    Json(HashMap::from([
        ("IMU", sensor_manager.check_imu()),
        ("Ultrasonic Sensor", sensor_manager.check_distance_sensor()),
        ("Camera", sensor_manager.check_camera()),
    ]))
}

async fn set_udp_sensor(Path(sensor): Path<String>) {
    // TODO Set the sensor that the Udp Sensor is sending
}

#[derive(serde::Deserialize)]
struct AccelerationAndSteering {
    pub acceleration: f64,
    pub steering: f64,
}

async fn set_steering_and_acceleration(
    State(state): State<GlobalState>,
    Json(values): Json<AccelerationAndSteering>,
) -> StatusCode {
    if *state.car_state.lock().await != CarStates::RemoteControlled {
        return StatusCode::BAD_REQUEST;
    }

    let motor = state.motor_manager.clone();

    task::spawn_blocking(move || {
        motor.set_acceleration(values.acceleration);
        motor.set_steering(values.steering);
    })
    .await
    .unwrap();

    StatusCode::OK
}

pub async fn http_server(global_state: GlobalState) -> std::io::Result<()> {
    let app = Router::new()
        .route("/state", get(get_current_state))
        .route("/state", post(set_current_state))
        .route("/available_sensors", get(get_all_available_sensors))
        .route("/udp_sensor", post(set_udp_sensor))
        .route("/remote_control", post(set_steering_and_acceleration))
        .with_state(global_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 80));
    let http_service = app.into_make_service();

    axum_server::bind(addr).serve(http_service).await
}
