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
use crate::http::control::PidManager;
use crate::http::udp_broadcast::UdpBroadcast;
use crate::sensors::manager::SensorManager;

mod actuator;
pub mod config;
mod control;
mod frontend;
mod sensor;
mod udp_broadcast;

/// Global state for the HTTP server
/// This is used to share state between the different routes
pub struct GlobalState {
    pub udp_manager: Arc<Mutex<UdpBroadcast>>,
    pub sensor_manager: Arc<Mutex<SensorManager>>,
    pub actuator_manager: Arc<ActuatorManager>,
    pub pids: Arc<PidManager>,
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
            udp_manager: UdpBroadcast::new(sensor_manager.clone())
                .expect("Failed to initialize UDP Manager"),
            sensor_manager,
            actuator_manager: Arc::new(actuator_manager),
            server_config: Arc::new(Mutex::new(server_config)),
            pids: Arc::new(PidManager::default()),
        }
    }
}

/// Starts the HTTP server
pub async fn http_server(global_state: GlobalState) -> std::io::Result<()> {
    let global_state = Arc::new(global_state);

    let api_router = Router::new()
        .route("/", get(|| async { "Server is online" }))
        .nest("/actuators", actuator::router())
        .nest("/control", control::router())
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
