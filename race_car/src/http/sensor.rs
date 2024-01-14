use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use tracing::info;

use crate::http::udp_broadcast::UdpActiveSensor;
use crate::http::GlobalState;
use crate::sensors::{Ambience, Gps, Imu, UltrasonicSensor};

pub fn router(global_state: Arc<GlobalState>) -> Router {
    Router::new()
        .route("/", get(get_all_available_sensors))
        .route("/active_udp", post(set_udp_sensor))
        .with_state(global_state)
}

async fn get_all_available_sensors(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    let mut sensor_manager = state.sensor_manager.lock().await;

    Json(HashMap::from([
        (Imu::NAME, sensor_manager.imu().is_some()),
        (
            UltrasonicSensor::NAME,
            sensor_manager.ultrasonic().is_some(),
        ),
        ("Camera", false),
        (Ambience::NAME, sensor_manager.ambience().is_some()),
        (Gps::NAME, sensor_manager.gps().is_some()),
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
                Imu::NAME => UdpActiveSensor::Imu,
                UltrasonicSensor::NAME => UdpActiveSensor::Ultrasonic,
                Gps::NAME => UdpActiveSensor::Gps,
                Ambience::NAME => UdpActiveSensor::Ambience,
                _ => return None,
            })
        })
        .collect();

    if sensors.iter().any(|sensor| sensor.is_none()) {
        return StatusCode::BAD_REQUEST;
    }

    let sensors = sensors.into_iter().flatten().collect();
    info!("Active Udp Sensors: {:?}", sensors);

    let mut sensor_manager = state.sensor_manager.lock().await;
    let mut udp_manager = state.udp_manager.lock().await;

    udp_manager.save_sensor_config(&mut sensor_manager);
    udp_manager.set_active_sensor(sensors, format!("{}:3001", addr.ip()));

    StatusCode::OK
}
