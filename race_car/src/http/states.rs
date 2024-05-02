//! HTTP routes for reading and changing the car's state.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use sensors::drivers::set_board_led_status;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, IntoStaticStr};
use tracing::error;

use crate::http::GlobalState;

/// The different states the car can be in.
/// - Standby: The default state of the car.
/// - Config: The car is in config mode. This means that the car will not drive and the sensors will be configured.
/// - RemoteControlled: The car is controlled by a remote.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    EnumIter,
    IntoStaticStr,
    AsRefStr,
)]
pub enum CarStates {
    #[default]
    Standby,
    Config,
    RemoteControlled,
}

/// Creates an object that manages all state related routes
pub fn router() -> Router<Arc<GlobalState>> {
    Router::new()
        .route("/all", get(get_all_states))
        .route("/", get(get_current_state))
        .route("/:new_state", post(set_current_state))
}

/// Returns a list of all available states
///
/// See [CarStates] for more information
async fn get_all_states() -> impl IntoResponse {
    Json(&[
        CarStates::Standby,
        CarStates::Config,
        CarStates::RemoteControlled,
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

    /*{
        let mut motors = state.motor_driver.lock().await;
        motors.stop_motor(Motor::Speed);
        motors.stop_motor(Motor::Steering);
    }*/

    {
        let mut udp = state.udp_manager.lock().await;
        udp.save_sensor_config(&mut sensor_manager);
        udp.set_config_mode(new_car_state == CarStates::Config);
    }

    set_board_led_status(false)
        .inspect_err(|e| error!("Failed to set board led: {e}"))
        .ok();

    match new_car_state {
        CarStates::Standby => sensor_manager.stop_listening_to_sensors(),
        CarStates::Config => sensor_manager.stop_listening_to_sensors(),
        CarStates::RemoteControlled => {
            // let state = state.clone();
            sensor_manager.start_listening_to_sensors();
            set_board_led_status(true).unwrap();
            // let start_time = SystemTime::now();
            // let receiver = sensor_manager.get_data_receiver().add_stream();
            set_board_led_status(false)
                .inspect_err(|e| error!("Failed to set board led: {e}"))
                .ok();

            /*std::thread::spawn(move || {
                let date = Local::now();

                let path = get_car_file(format!("{}.log", date.format("%Y-%m-%d_%H-%M-%S")));
                let mut output_file = File::create(path).unwrap();

                output_file
                    .write_all(
                        format!(
                            "{}\n",
                            start_time.duration_since(UNIX_EPOCH).unwrap().as_millis()
                        )
                        .as_bytes(),
                    )
                    .unwrap();

                while let Ok(data) = receiver.recv() {
                    if *state.car_state.blocking_lock() != CarStates::RemoteControlled {
                        break;
                    }

                    output_file
                        .write_all(
                            format!("{}\n", serde_json::to_string(&data).unwrap()).as_bytes(),
                        )
                        .unwrap();
                }
                output_file.sync_data().unwrap();
                info!("Log thread stopped")
            });*/
        }
    }

    // state.pids.reset().await;

    StatusCode::OK
}
