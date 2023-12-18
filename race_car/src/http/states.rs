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

#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CarStates {
    #[default]
    Standby,
    Config,
    RemoteControlled,
    AutonomousControlled,
}

pub fn router(global_state: Arc<GlobalState>) -> Router {
    Router::new()
        .route("/all", get(get_all_states))
        .route("/", get(get_current_state))
        .route("/:new_state", post(set_current_state))
        .with_state(global_state)
}

async fn get_all_states() -> impl IntoResponse {
    Json(&[
        CarStates::Standby,
        CarStates::Config,
        CarStates::RemoteControlled,
        CarStates::AutonomousControlled,
    ])
}

async fn get_current_state(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    Json(*state.car_state.lock().await)
}

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

    match new_car_state {
        CarStates::Standby => sensor_manager.reset(),
        CarStates::Config => sensor_manager.reset(),
        CarStates::RemoteControlled => {
            let receiver = sensor_manager.listen_to_all_sensors();
            // return StatusCode::OK; // TODO

            std::thread::spawn(move || {
                let date = Local::now();

                let path = get_car_file(format!("{}.log", date.format("%Y.%m.%d_%H:%M:%S")));
                let mut output_file = File::create(path).unwrap();

                while let Ok(data) = receiver.recv() {
                    output_file
                        .write_all(serde_json::to_string(&data).unwrap().as_bytes())
                        .unwrap();
                    output_file.write_all(&[b'\n']).unwrap();
                }
                output_file.sync_data().unwrap();
                info!("Stopping log thread")
            });
        } // TODO Do something with it
        CarStates::AutonomousControlled => {
            sensor_manager.listen_to_all_sensors();
        }
    }

    StatusCode::OK
}
