//! HTTP car server.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use tokio::sync::Mutex;
use tower_http::trace;
use tower_http::trace::TraceLayer;
use tracing::Level;

use crate::actuators::manager::ActuatorManager;
use crate::http::config::ServerConfig;
use crate::http::states::CarStates;
use crate::http::udp_broadcast::UdpBroadcast;
use crate::sensors::manager::SensorManager;

mod actuator;
pub mod config;
mod control;
mod frontend;
mod sensor;
mod states;
mod udp_broadcast;

/// Global state for the HTTP server
/// This is used to share state between the different routes
pub struct GlobalState {
    pub car_state: Mutex<CarStates>,
    pub udp_manager: Arc<Mutex<UdpBroadcast>>,
    pub sensor_manager: Arc<Mutex<SensorManager>>,
    pub actuator_manager: Arc<ActuatorManager>,
    // pub pids: Arc<PidManager>,
    pub server_config: Arc<Mutex<ServerConfig>>,
}

impl GlobalState {
    pub fn new(
        sensor_manager: SensorManager,
        actuator_manager: ActuatorManager,
        server_config: ServerConfig,
    ) -> Self {
        let sensor_manager = Arc::new(Mutex::new(sensor_manager));
        Self {
            car_state: Mutex::default(),
            udp_manager: UdpBroadcast::new(sensor_manager.clone())
                .expect("Failed to initialize UDP Manager"),
            sensor_manager,
            actuator_manager: Arc::new(actuator_manager),
            server_config: Arc::new(Mutex::new(server_config)),
            // motor_driver: Arc::new(Mutex::new(motor_driver)),
            // pids: Arc::new(PidManager::new(
            //     PidController::new(1.0, 0.0, 0.0),
            //     PidController::new(1.0, 0.0, 0.3)
            //         .set_input_range(-1.0, 1.0)
            //         .set_output_range(-0.9, 0.9),
            // )),
        }
    }
}

/// Starts the HTTP server
pub async fn http_server(global_state: GlobalState) -> std::io::Result<()> {
    let global_state = Arc::new(global_state);

    let api_router = Router::new()
        .route("/", get(|| async { "Server is online" }))
        .nest("/actuators", actuator::router())
        .nest("/state", states::router())
        // .nest("/api/control", control::router())
        .nest("/sensors", sensor::router());

    let app = Router::new()
        .merge(frontend::router())
        .nest("/api", api_router)
        .with_state(global_state)
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
