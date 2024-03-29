//! HTTP car server.

use std::fs::File;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::http::control::PidManager;
use axum::routing::get;
use axum::Router;
use chrono::Local;
use shared::math::pid::PidController;
use tokio::sync::Mutex;
use tower_http::trace;
use tower_http::trace::TraceLayer;
use tracing::Level;

use crate::http::states::CarStates;
use crate::http::udp_broadcast::UdpBroadcast;
use crate::sensors::manager::SensorManager;
use crate::sensors::motor_driver::MotorDriver;
use crate::utils::files::get_car_file;

mod control;
mod motor;
mod sensor;
mod states;
mod udp_broadcast;

/// Global state for the HTTP server
/// This is used to share state between the different routes
pub struct GlobalState {
    pub car_state: Mutex<CarStates>,
    pub udp_manager: Arc<Mutex<UdpBroadcast>>,
    pub sensor_manager: Arc<Mutex<SensorManager>>,
    pub motor_driver: Arc<Mutex<MotorDriver>>,
    pub pids: Arc<PidManager>,
    pub motor_file: Mutex<File>,
}

impl GlobalState {
    pub fn new(sensor_manager: SensorManager, motor_driver: MotorDriver) -> Self {
        let date = Local::now();
        let sensor_manager = Arc::new(Mutex::new(sensor_manager));
        Self {
            car_state: Mutex::default(),
            udp_manager: UdpBroadcast::new(sensor_manager.clone())
                .expect("Failed to initialize UDP Manager"),
            sensor_manager,
            motor_driver: Arc::new(Mutex::new(motor_driver)),
            pids: Arc::new(PidManager::new(
                PidController::new(1.0, 0.0, 0.0),
                PidController::new(1.0, 0.0, 0.3)
                    .set_input_range(-1.0, 1.0)
                    .set_output_range(-0.9, 0.9),
            )),
            motor_file: Mutex::new(
                File::create(get_car_file(format!("{}.motor", date.format("%H-%M-%S")))).unwrap(),
            ),
        }
    }
}

/// Starts the HTTP server
pub async fn http_server(global_state: GlobalState) -> std::io::Result<()> {
    let global_state = Arc::new(global_state);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/motors", motor::router(global_state.clone()).await)
        .nest("/state", states::router(global_state.clone()))
        .nest("/sensors", sensor::router(global_state.clone()))
        .nest("/control", control::router(global_state))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    const PORT: u16 = 8080;

    let addr = SocketAddr::from(([0, 0, 0, 0], PORT));
    let http_service = app.into_make_service_with_connect_info::<SocketAddr>();

    println!("Server started on port {PORT}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, http_service).await
}
