//! HTTP routes for reading and changing the car's state.

use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use crate::files::get_car_file;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Local;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::http::GlobalState;
use crate::sensors::motor_driver::Motor;

/// The different states the car can be in.
/// - Standby: The default state of the car.
/// - Config: The car is in config mode. This means that the car will not drive and the sensors will be configured.
/// - RemoteControlled: The car is controlled by a remote.
#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CarStates {
    #[default]
    Standby,
    Config,
    RemoteControlled,
    AutonomousControlled,
}

/// Creates an object that manages all state related routes
pub fn router(global_state: Arc<GlobalState>) -> Router {
    Router::new()
        .route("/all", get(get_all_states))
        .route("/", get(get_current_state))
        .route("/:new_state", post(set_current_state))
        .with_state(global_state)
}

/// Returns a list of all available states
///
/// See [CarStates] for more information
async fn get_all_states() -> impl IntoResponse {
    Json(&[
        CarStates::Standby,
        CarStates::Config,
        CarStates::RemoteControlled,
        CarStates::AutonomousControlled,
    ])
}

/// Returns the current state of the car
///
/// See [CarStates] for more information
async fn get_current_state(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    Json(*state.car_state.lock().await)
}

/// Sets the current state of the car
///
/// See [CarStates] for more information
async fn set_current_state(
    State(state): State<Arc<GlobalState>>,
    Path(new_car_state): Path<CarStates>,
) -> StatusCode {
    let mut car_state_guard = state.car_state.lock().await;

    if *car_state_guard == new_car_state {
        return StatusCode::OK;
    }

    *car_state_guard = new_car_state;

    let mut sensor_manager = state.sensor_manager.lock().await;

    {
        let mut motors = state.motor_driver.lock().await;
        motors.stop_motor(Motor::Acceleration);
        motors.stop_motor(Motor::Steering);
    }

    {
        let mut udp = state.udp_manager.lock().await;
        udp.save_sensor_config(&mut sensor_manager);
        udp.set_config_mode(new_car_state == CarStates::Config);
    }

    match new_car_state {
        CarStates::Standby => {
            sensor_manager.reset();
        }
        CarStates::Config => {
            sensor_manager.reset();
        }
        CarStates::RemoteControlled => {
            let state = state.clone();
            let receiver = sensor_manager.listen_to_all_sensors();

            std::thread::spawn(move || {
                let date = Local::now();

                let path = get_car_file(format!("{}.log", date.format("%Y.%m.%d_%H:%M:%S")));
                let mut output_file = File::create(path).unwrap();

                while let Ok(data) = receiver.recv() {
                    if *state.car_state.blocking_lock() != CarStates::RemoteControlled {
                        break;
                    }
                    output_file
                        .write_all(serde_json::to_string(&data).unwrap().as_bytes())
                        .unwrap();
                    output_file.write_all(&[b'\n']).unwrap();
                }
                output_file.sync_data().unwrap();
                info!("Log thread stopped")
            });
        } // TODO Do something with it
        CarStates::AutonomousControlled => {
            sensor_manager.listen_to_all_sensors();
        }
    }

    state.pids.reset().await;

    StatusCode::OK
}
