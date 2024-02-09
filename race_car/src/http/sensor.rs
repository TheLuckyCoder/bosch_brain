//! HTTP routes for interacting with the car's sensors.

use std::collections::{BTreeMap, HashMap};
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
use crate::sensors::{AmbienceSensor, Gps, Imu, UltrasonicSensor};

/// Creates an object that manages all the sensor routes
pub fn router(global_state: Arc<GlobalState>) -> Router {
    Router::new()
        .route("/", get(get_all_available_sensors))
        .route("/active_udp", post(set_udp_sensors))
        .with_state(global_state)
}

/// Returns a list of all available and initialized sensors
async fn get_all_available_sensors(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    let mut sensor_manager = state.sensor_manager.lock().await;

    Json(BTreeMap::from([
        (Imu::NAME, sensor_manager.imu().is_some()),
        (
            UltrasonicSensor::NAME,
            sensor_manager.ultrasonic().is_some(),
        ),
        ("Camera", false),
        (AmbienceSensor::NAME, sensor_manager.ambience().is_some()),
        (Gps::NAME, sensor_manager.gps().is_some()),
    ]))
}

/// Sets the active UDP sensors from which data will be streamed on the UDP port.
///
/// This will do different things depending on the state the car is in:
/// - Standby: Will do nothing
/// - Config: Will send the config data of the sensors
/// - RemoteControlled: Will send the data of the sensors
///
/// See [UdpActiveSensor] too see which sensors are available.
///
async fn set_udp_sensors(
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
                AmbienceSensor::NAME => UdpActiveSensor::Ambience,
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
