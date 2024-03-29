//! HTTP routes for manually controlling the car's motors.

use crate::http::states::CarStates;
use crate::http::GlobalState;
use crate::sensors::motor_driver::{Motor, MotorParams};
use crate::utils::files::get_car_dir;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::task;
use tokio::time::sleep;
use tracing::{info, log};

const ALL_MOTORS: [Motor; 2] = [Motor::Steering, Motor::Speed];

fn get_motor_params_file(motor: Motor) -> PathBuf {
    let mut path = get_car_dir();
    path.push(&format!("motor_params_{motor:?}.json"));
    path
}

async fn read_params_from_files(global_state: &Arc<GlobalState>) {
    for motor in ALL_MOTORS {
        let file_path = get_motor_params_file(motor);
        let params_file = match std::fs::File::open(&file_path) {
            Ok(params_file) => params_file,
            Err(e) => {
                log::warn!("Failed to read {} reason: {e}", file_path.display());
                continue;
            }
        };
        let mut reader = BufReader::new(params_file);

        match serde_json::from_reader(&mut reader) {
            Ok(params) => global_state
                .motor_driver
                .lock()
                .await
                .set_params(motor, params),
            Err(e) => {
                log::error!("Failed to deserialize {} reason: {e}", file_path.display());
            }
        }
    }
}

/// Creates an object that manages all the motor routes
pub async fn router(global_state: Arc<GlobalState>) -> Router {
    read_params_from_files(&global_state).await;

    Router::new()
        .route("/", get(get_motors))
        .route("/params/:motor", get(get_motor_parameters))
        .route("/params/:motor", post(set_motor_parameters))
        .route("/stop/:motor", post(stop_motor))
        .route("/stop", post(stop_motor))
        .route("/set/:motor/:value", post(set_motor_value))
        .route("/", post(set_all_motors))
        .route("/pause/:motor", post(pause_motor))
        .route("/pause", post(pause_motor))
        .route("/resume/:motor", post(resume_motor))
        .route("/resume", post(resume_motor))
        .route("/sweep/:motor", post(motor_sweep))
        .with_state(global_state)
}

/// Returns a list of all motors
async fn get_motors() -> impl IntoResponse {
    Json(ALL_MOTORS)
}

/// Returns the current parameters for the given motor
async fn get_motor_parameters(
    State(state): State<Arc<GlobalState>>,
    Path(motor): Path<Motor>,
) -> impl IntoResponse {
    let motor_driver = state.motor_driver.lock().await;

    Json(motor_driver.get_params(motor))
}

/// Sets the parameters for the given motor
async fn set_motor_parameters(
    State(state): State<Arc<GlobalState>>,
    Path(motor): Path<Motor>,
    Json(params): Json<MotorParams>,
) {
    let mut motor_driver = state.motor_driver.lock().await;

    info!("Motor params received: {params:?}");
    motor_driver.set_params(motor, params.clone());

    let motor_params_path = get_motor_params_file(motor);

    task::spawn_blocking(move || {
        let file = std::fs::File::create(motor_params_path).unwrap();
        let mut writer = BufWriter::new(file);

        serde_json::to_writer(&mut writer, &params).unwrap();
        writer.flush().unwrap();
        writer.get_ref().sync_all().unwrap();
    })
    .await
    .unwrap();
}

/// Stops the given motor, or all motors if no motor is specified
async fn stop_motor(State(state): State<Arc<GlobalState>>, motor: Option<Path<Motor>>) {
    let mut motor_driver = state.motor_driver.lock().await;

    if let Some(Path(motor)) = motor {
        motor_driver.stop_motor(motor);
    } else {
        for motor in ALL_MOTORS {
            motor_driver.stop_motor(motor);
        }
    }
}

/// Sets the value for the given motor
async fn set_motor_value(
    State(state): State<Arc<GlobalState>>,
    Path((motor, value)): Path<(Motor, f64)>,
) {
    let mut motor_driver = state.motor_driver.lock().await;

    motor_driver.set_motor_value(motor, value);
}

/// Pauses the given motor, or all motors if no motor is specified
async fn pause_motor(State(state): State<Arc<GlobalState>>, motor: Option<Path<Motor>>) {
    let mut motor_driver = state.motor_driver.lock().await;

    if let Some(Path(motor)) = motor {
        motor_driver.pause_motor(motor);
    } else {
        for motor in ALL_MOTORS {
            motor_driver.pause_motor(motor);
        }
    }
}

/// Resumes the given motor, or all motors if no motor is specified
async fn resume_motor(State(state): State<Arc<GlobalState>>, motor: Option<Path<Motor>>) {
    let mut motor_driver = state.motor_driver.lock().await;

    if let Some(Path(motor)) = motor {
        motor_driver.resume_motor(motor);
    } else {
        for motor in ALL_MOTORS {
            motor_driver.resume_motor(motor);
        }
    }
}

async fn motor_sweep(State(state): State<Arc<GlobalState>>, Path(motor): Path<Motor>) {
    if *state.car_state.lock().await != CarStates::Config {
        return;
    }

    tokio::spawn(async move {
        let mut motor_driver = state.motor_driver.lock().await;

        for i in 0..10 {
            motor_driver.set_motor_value(motor, i as f64 / 10f64);
            sleep(Duration::from_millis(150)).await;
        }

        for i in -10..=10 {
            motor_driver.set_motor_value(motor, -i as f64 / 10f64);
            sleep(Duration::from_millis(150)).await;
        }
        for i in 0..=10 {
            motor_driver.set_motor_value(motor, (-10 + i) as f64 / 10f64);
            sleep(Duration::from_millis(150)).await;
        }

        motor_driver.stop_motor(motor);
    });
}

#[derive(Default, Serialize, Deserialize)]
struct SpeedAndSteering {
    pub speed: f64,
    pub steering: f64,
}

/// Sets the value for both the acceleration and steering motors at once
async fn set_all_motors(
    State(state): State<Arc<GlobalState>>,
    values: Option<Json<SpeedAndSteering>>,
) -> StatusCode {
    if *state.car_state.lock().await != CarStates::RemoteControlled {
        return StatusCode::UNAUTHORIZED;
    }
    let mut motor_file = state.motor_file.lock().await;

    let Json(values) = values.unwrap_or_default();

    let mut motor = state.motor_driver.lock().await;

    info!(
        "Motor Values received: Speed({}) Steering({})",
        values.speed, values.steering
    );

    motor.set_motor_value(Motor::Speed, values.speed);
    motor.set_motor_value(Motor::Steering, values.steering);
    motor_file
        .write_all(serde_json::to_string(&values).unwrap().as_ref())
        .expect("Failed to write to file");

    StatusCode::OK
}
