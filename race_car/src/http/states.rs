use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::time::SystemTime;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::http::{get_home_dir, GlobalState};

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
        .route("/current", get(get_current_state))
        .route("/current/:new_state", post(set_current_state))
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

            task::spawn_blocking(move || {
                let system_time = SystemTime::now();
                let mut path = get_home_dir();
                path.push(format!(
                    "{}.log",
                    system_time
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                ));
                let mut output_file = File::create(path).unwrap();

                while let Ok(data) = receiver.recv() {
                    output_file
                        .write_all(serde_json::to_string(&data).unwrap().as_bytes())
                        .unwrap();
                    output_file.write(&[b'\n']).unwrap();
                }
            });
        } // TODO Do something with it
        CarStates::AutonomousControlled => {
            sensor_manager.listen_to_all_sensors();
        }
    }

    StatusCode::OK
}
