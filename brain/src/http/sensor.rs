use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use axum::extract::{ConnectInfo, State};
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::routing::{get, post};
use crate::http::GlobalState;
use crate::http::udp_manager::UdpActiveSensor;

pub fn router(global_state: Arc<GlobalState>) -> Router {
    Router::new()
        .route("/", get(get_all_available_sensors))
        .route("/active_udp", post(set_udp_sensor))
        .with_state(global_state)
}

async fn get_all_available_sensors(
    State(state): State<Arc<GlobalState>>,
) -> impl IntoResponse {
    let sensor_manager = &state.sensor_manager;

    Json(HashMap::from([
        ("IMU", sensor_manager.check_imu()),
        ("Ultrasonic", sensor_manager.check_distance_sensor()),
        ("Camera", sensor_manager.check_camera()),
        ("Temperature", false),
        ("GPS", false),
    ]))
}

async fn set_udp_sensor(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<GlobalState>>,
    Json(sensor): Json<Vec<String>>,
) -> StatusCode {
    let sensors: Vec<_> = sensor
        .into_iter()
        .map(|sensor| {
            Some(match sensor.as_str() {
                "IMU" => UdpActiveSensor::Imu,
                "Ultrasonic" => UdpActiveSensor::Ultrasonic,
                _ => return None,
            })
        })
        .collect();

    if sensors.iter().any(|sensor| sensor.is_none()) {
        return StatusCode::BAD_REQUEST;
    }

    let sensors = sensors.into_iter().filter_map(|sensor| sensor).collect();

    state
        .udp_manager
        .set_active_sensor(sensors, format!("{}:3001", addr.ip()));

    StatusCode::OK
}
