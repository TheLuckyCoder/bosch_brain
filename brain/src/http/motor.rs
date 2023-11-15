use crate::http::states::CarStates;
use crate::http::GlobalState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use sensors::{Motor, MotorParams};
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

const ALL_MOTORS: [Motor; 2] = [Motor::Steering, Motor::Acceleration];

fn get_home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir().expect("Failed to get current working directory"))
}

fn get_motor_params_file(motor: Motor) -> PathBuf {
    get_home_dir().with_file_name(format!("motor_params_{motor:?}.json"))
}

fn read_params_from_files(global_state: &Arc<GlobalState>) {
    for motor in ALL_MOTORS {
        let file_path = get_motor_params_file(motor);
        let params_file = match std::fs::File::open(&file_path) {
            Ok(params_file) => params_file,
            Err(_) => continue,
        };
        let mut reader = BufReader::new(params_file);

        let params: MotorParams = serde_json::from_reader(&mut reader)
            .unwrap_or_else(|_| panic!("Failed to deserialize {}", file_path.display()));

        global_state
            .motor_driver
            .blocking_lock()
            .set_params(motor, params)
    }
}

pub fn router(global_state: Arc<GlobalState>) -> Router {
    read_params_from_files(&global_state);

    Router::new()
        .route("/all", get(get_motors))
        .route("/params/:motor", get(get_motor_parameters))
        .route("/params/:motor", post(set_motor_parameters))
        .route("/stop/:motor", post(stop_motor))
        .route("/set/:motor", post(set_motor_value))
        .route("/set_all", post(set_all_motors))
        .route("/pause/:motor", post(pause_motor))
        .route("/resume/:motor", post(resume_motor))
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

    motor_driver.set_params(motor, params.clone());

    let motor_params_path = get_motor_params_file(motor);

    let file = std::fs::File::create(motor_params_path).unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &params).unwrap();
    writer.flush().unwrap();
}

async fn stop_motor(State(state): State<Arc<GlobalState>>, Path(motor): Path<Motor>) {
    let mut motor_driver = state.motor_driver.lock().await;

    motor_driver.stop_motor(motor);
}

async fn set_motor_value(
    State(state): State<Arc<GlobalState>>,
    Path(motor): Path<Motor>,
    Json(value): Json<f64>,
) {
    let mut motor_driver = state.motor_driver.lock().await;

    motor_driver.set_motor_value(motor, value);
}

async fn pause_motor(State(state): State<Arc<GlobalState>>, Path(motor): Path<Motor>) {
    let mut motor_driver = state.motor_driver.lock().await;

    motor_driver.pause_motor(motor);
}

async fn resume_motor(State(state): State<Arc<GlobalState>>, Path(motor): Path<Motor>) {
    let mut motor_driver = state.motor_driver.lock().await;

    motor_driver.resume_motor(motor);
}

async fn motor_sweep(State(state): State<Arc<GlobalState>>, Path(motor): Path<Motor>) {
    tokio::spawn(async move {
        let mut motor_driver = state.motor_driver.lock().await;

        for i in 0..10 {
            motor_driver.set_motor_value(motor, i as f64 / 10f64);
            std::thread::sleep(Duration::from_millis(100));
        }

        for i in -10..=10 {
            motor_driver.set_motor_value(motor, -i as f64 / 10f64);
            std::thread::sleep(Duration::from_millis(100));
        }

        for i in 0..=10 {
            motor_driver.set_motor_value(motor, -i as f64 / 10f64);
            std::thread::sleep(Duration::from_millis(100));
        }
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

    motor.set_motor_value(Motor::Acceleration, values.acceleration);
    motor.set_motor_value(Motor::Steering, values.steering);

    StatusCode::OK
}
