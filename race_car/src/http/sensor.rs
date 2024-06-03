//! HTTP routes for interacting with the car's sensors.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Json, Router};
use axum::extract::{ConnectInfo, Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use strum::IntoEnumIterator;
use tokio::task;
use tracing::info;

use sensors::name::SensorName;
use sensors::SensorData;

use crate::http::GlobalState;

/// Creates an object that manages all the sensor routes
pub fn router() -> Router<Arc<GlobalState>> {
    Router::new()
        .route("/", get(get_sensors))
        .route("/active", get(get_active_sensors))
        .route("/active_udp", post(set_udp_sensors))
        .route("/read/:name", get(read_sensor))
}

/// Returns a list of all registered sensors
async fn get_sensors() -> impl IntoResponse {
    Json(SensorName::iter().collect::<Vec<_>>())
}

/// Returns a list of all active sensors
async fn get_active_sensors(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    Json(state.actuator_manager.get_active_actuators().into_iter().map(|(name, _)| name).collect::<Vec<_>>())
}

async fn read_sensor(
    State(state): State<Arc<GlobalState>>,
    Path(name): Path<SensorName>,
) -> impl IntoResponse {
    task::spawn_blocking(move || {
        let sensor_manager = state.sensor_manager.blocking_lock();

        let mut sensor = sensor_manager.get_sensor(&name).unwrap().lock().unwrap();
        Json(sensor.read_data())
    }).await.unwrap()
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
    udp_manager.set_active_sensors(sensors, format!("{}:3001", addr.ip()));

    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use reqwest;

    use sensors::{BasicSensor, SensorData};
    use sensors::name::SensorName;

    use crate::actuators::manager::ActuatorManager;
    use crate::http::{GlobalState, http_server};
    use crate::http::config::ServerConfig;
    use crate::sensors::manager::SensorManager;

    struct TestSensor {}

    impl BasicSensor for TestSensor {
        fn name(&self) -> SensorName {
            SensorName::Velocity
        }

        fn read_data(&mut self) -> SensorData {
            SensorData::Velocity(20.0)
        }
    }

    #[tokio::test]
    async fn read_sensor() {
        let client = reqwest::Client::new();
        let mut sensor_manager = SensorManager::new();
        sensor_manager.add_sensor(TestSensor {});

        tokio::task::spawn(async {
            http_server(GlobalState::new(
                sensor_manager,
                ActuatorManager::new(),
                ServerConfig::default(),
            ))
            .await
        });

        // Wait for the server to start
        tokio::time::sleep(Duration::from_millis(1)).await;

        let response = client
            .get("http://localhost:8080/api/sensors/Velocity/read")
            .send()
            .await
            .unwrap();

        let status_code = response.status();
        let text = response.text().await.unwrap();
        println!("Status: {}; {}", status_code, text);
        let json: SensorData = serde_json::from_str(&text).unwrap();

        assert_eq!(json, SensorData::Velocity(20.0));
    }
}
