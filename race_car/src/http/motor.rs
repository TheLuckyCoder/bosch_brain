use crate::http::states::CarStates;
use crate::http::{get_home_dir, GlobalState};
use crate::sensors::{Motor, MotorParams};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::task;
use tokio::time::sleep;
use tracing::{info, log};

const ALL_MOTORS: [Motor; 2] = [Motor::Steering, Motor::Acceleration];

fn get_motor_params_file(motor: Motor) -> PathBuf {
    let mut path = get_home_dir();
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

pub async fn router(global_state: Arc<GlobalState>) -> Router {
    read_params_from_files(&global_state).await;

    Router::new()
        .route("/all", get(get_motors))
        .route("/params/:motor", get(get_motor_parameters))
        .route("/params/:motor", post(set_motor_parameters))
        .route("/stop/:motor", post(stop_motor))
        .route("/set/:motor/:value", post(set_motor_value))
        .route("/set_all", post(set_all_motors))
        .route("/pause/:motor", post(pause_motor))
        .route("/pause", post(pause_motors))
        .route("/resume/:motor", post(resume_motor))
        .route("/resume", post(resume_motors))
        .route("/sweep/:motor", post(motor_sweep))
        .with_state(global_state)
}

async fn get_motors() -> impl IntoResponse {
    Json(ALL_MOTORS)
}

async fn get_motor_parameters(
    State(state): State<Arc<GlobalState>>,
    Path(motor): Path<Motor>,
) -> impl IntoResponse {
    let motor_driver = state.motor_driver.lock().await;

    Json(motor_driver.get_params(motor))
}

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

async fn stop_motor(State(state): State<Arc<GlobalState>>, Path(motor): Path<Motor>) {
    let mut motor_driver = state.motor_driver.lock().await;

    motor_driver.stop_motor(motor);
}

async fn set_motor_value(
    State(state): State<Arc<GlobalState>>,
    Path((motor, value)): Path<(Motor, f64)>,
) {
    let mut motor_driver = state.motor_driver.lock().await;

    motor_driver.set_motor_value(motor, value);
}

async fn pause_motor(State(state): State<Arc<GlobalState>>, Path(motor): Path<Motor>) {
    let mut motor_driver = state.motor_driver.lock().await;

    motor_driver.pause_motor(motor);
}

async fn pause_motors(
    State(state): State<Arc<GlobalState>>,
    Json(motors): Json<Vec<Motor>>,
) -> StatusCode {
    let mut motor_driver = state.motor_driver.lock().await;

    for motor in motors {
        motor_driver.pause_motor(motor);
    }

    StatusCode::OK
}

async fn resume_motor(State(state): State<Arc<GlobalState>>, Path(motor): Path<Motor>) {
    let mut motor_driver = state.motor_driver.lock().await;

    motor_driver.resume_motor(motor);
}

async fn resume_motors(
    State(state): State<Arc<GlobalState>>,
    Json(motors): Json<Vec<Motor>>,
) -> StatusCode {
    let mut motor_driver = state.motor_driver.lock().await;

    for motor in motors {
        motor_driver.resume_motor(motor);
    }

    StatusCode::OK
}

async fn motor_sweep(State(state): State<Arc<GlobalState>>, Path(motor): Path<Motor>) {
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

#[derive(serde::Deserialize)]
struct AccelerationAndSteering {
    pub acceleration: f64,
    pub steering: f64,
}

async fn set_all_motors(
    State(state): State<Arc<GlobalState>>,
    Json(values): Json<AccelerationAndSteering>,
) -> StatusCode {
    if *state.car_state.lock().await != CarStates::RemoteControlled {
        return StatusCode::BAD_REQUEST;
    }

    let mut motor = state.motor_driver.lock().await;

    info!(
        "Motor Values received: Acceleration({}) Steering({})",
        values.acceleration, values.steering
    );

    motor.set_motor_value(Motor::Acceleration, values.acceleration);
    motor.set_motor_value(Motor::Steering, values.steering);

    StatusCode::OK
}
