//! HTTP routes for interacting with the car's sensors.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Json, Router};
use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use strum::IntoEnumIterator;
use tracing::info;

use crate::http::GlobalState;
use crate::sensors::SensorName;

/// Creates an object that manages all the sensor routes
pub fn router(global_state: Arc<GlobalState>) -> Router {
    Router::new()
        .route("/", get(get_all_available_sensors))
        .route("/active_udp", post(set_udp_sensors))
        .with_state(global_state)
}

/// Returns a list of all available and initialized sensors
async fn get_all_available_sensors(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    let sensor_manager = state.sensor_manager.lock().await;

    let map: BTreeMap<&str, bool> = SensorName::iter().map(|sensor_name| {
        (sensor_name.into(), sensor_manager.get_sensor(&sensor_name).is_some())
    }).collect();

    Json(map)
}

/// Sets the active UDP sensors from which data will be streamed on the UDP port.
///
/// This will do different things depending on the state the car is in:
/// - Standby: Will do nothing
/// - Config: Will send the config data of the sensors
/// - RemoteControlled: Will send the data of the sensors
///
/// See [SensorName] to see which sensors are available.
///
async fn set_udp_sensors(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<GlobalState>>,
    Json(sensors): Json<Vec<SensorName>>,
) -> StatusCode {
    info!("Active Udp Sensors: {:?}", sensors);

    let mut sensor_manager = state.sensor_manager.lock().await;
    let mut udp_manager = state.udp_manager.lock().await;

    udp_manager.save_sensor_config(&mut sensor_manager);
    udp_manager.set_active_sensor(sensors, format!("{}:3001", addr.ip()));

    StatusCode::OK
}
