use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tower_http::trace;
use tower_http::trace::TraceLayer;
use tracing::Level;

use sensors::{MotorDriver, SensorManager};

use crate::udp_manager::{UdpActiveSensor, UdpManager};

#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CarStates {
    #[default]
    Standby,
    Config,
    RemoteControlled,
    AutonomousControlled,
}

pub struct GlobalState {
    pub car_state: Mutex<CarStates>,
    pub udp_manager: Arc<UdpManager>,
    pub sensor_manager: Arc<SensorManager>,
    pub motor_driver: Arc<Mutex<MotorDriver>>,
}

impl GlobalState {
    pub fn new(sensor_manager: Arc<SensorManager>, motor_driver: MotorDriver) -> Self {
        Self {
            car_state: Mutex::default(),
            udp_manager: UdpManager::new(sensor_manager.clone())
                .expect("Failed to initialize UDP Manager"),
            sensor_manager,
            motor_driver: Arc::new(Mutex::new(motor_driver)),
        }
    }
}

async fn get_all_states() -> Json<&'static [CarStates]> {
    Json(&[
        CarStates::Standby,
        CarStates::Config,
        CarStates::RemoteControlled,
        CarStates::AutonomousControlled,
    ])
}

async fn get_current_state(State(state): State<Arc<GlobalState>>) -> Json<CarStates> {
    Json(*state.car_state.lock().await)
}

async fn set_current_state(
    State(state): State<Arc<GlobalState>>,
    Path(new_car_state): Path<CarStates>,
) -> StatusCode {
    let mut car_state_guard = state.car_state.lock().await;

    *car_state_guard = new_car_state;

    StatusCode::OK
}

async fn get_all_available_sensors(
    State(state): State<Arc<GlobalState>>,
) -> Json<HashMap<&'static str, bool>> {
    let sensor_manager = &state.sensor_manager;

    Json(HashMap::from([
        ("IMU", sensor_manager.check_imu()),
        ("Ultrasonic", sensor_manager.check_distance_sensor()),
        ("Camera", sensor_manager.check_camera()),
        ("Temperature", false),
        ("GPS", false),
    ]))
}

async fn set_udp_sensor(
    State(state): State<Arc<GlobalState>>,
    Json(sensor): Json<Vec<String>>,
) -> StatusCode {
    let sensors: Vec<_> = sensor
        .into_iter()
        .map(|sensor| {
            Some(match sensor.as_str() {
                "Imu" => UdpActiveSensor::Imu,
                "Distance" => UdpActiveSensor::Distance,
                _ => return None,
            })
        })
        .collect();

    if sensors.iter().any(|sensor| sensor.is_none()) {
        return StatusCode::BAD_REQUEST;
    }

    state
        .udp_manager
        .set_active_sensor(sensors.into_iter().filter_map(|sensor| sensor).collect());

    StatusCode::OK
}

#[derive(serde::Deserialize)]
struct AccelerationAndSteering {
    pub acceleration: f64,
    pub steering: f64,
}

async fn set_steering_and_acceleration(
    State(state): State<Arc<GlobalState>>,
    Json(values): Json<AccelerationAndSteering>,
) -> StatusCode {
    if *state.car_state.lock().await != CarStates::RemoteControlled {
        return StatusCode::BAD_REQUEST;
    }

    let mut motor = state.motor_driver.lock().await;

    motor.set_acceleration(values.acceleration);
    motor.set_steering_angle(values.steering);

    StatusCode::OK
}

pub async fn http_server(global_state: GlobalState) -> std::io::Result<()> {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/states", get(get_all_states))
        .route("/current_state", get(get_current_state))
        .route("/current_state/:new_state", post(set_current_state))
        .route("/available_sensors", get(get_all_available_sensors))
        .route("/udp_sensor", post(set_udp_sensor))
        .route("/remote_control", post(set_steering_and_acceleration))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(Arc::new(global_state));

    const PORT: u16 = 8080;

    let addr = SocketAddr::from(([0, 0, 0, 0], PORT));
    let http_service = app.into_make_service();

    println!("Server started on port {PORT}");

    axum_server::bind(addr).serve(http_service).await
}
