use std::net::SocketAddr;
use std::sync::Arc;

use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use tokio::sync::Mutex;
use tower_http::trace;
use tower_http::trace::TraceLayer;
use tracing::Level;

use sensors::{MotorDriver, SensorManager};

use crate::http::states::CarStates;
use crate::http::udp_manager::UdpManager;

mod motor;
mod sensor;
mod states;
mod udp_manager;

pub struct GlobalState {
    pub car_state: Mutex<CarStates>,
    pub udp_manager: Arc<UdpManager>,
    pub sensor_manager: Arc<SensorManager>,
    pub motor_driver: Mutex<MotorDriver>,
}

impl GlobalState {
    pub fn new(sensor_manager: Arc<SensorManager>, motor_driver: MotorDriver) -> Self {
        Self {
            car_state: Mutex::default(),
            udp_manager: UdpManager::new(sensor_manager.clone())
                .expect("Failed to initialize UDP Manager"),
            sensor_manager,
            motor_driver: Mutex::new(motor_driver),
        }
    }
}

pub async fn http_server(global_state: GlobalState) -> std::io::Result<()> {
    let global_state = Arc::new(global_state);

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/motor", motor::router(global_state.clone()))
        .nest("/state", states::router(global_state.clone()))
        .nest("/sensors", sensor::router(global_state))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    const PORT: u16 = 8080;

    let addr = SocketAddr::from(([0, 0, 0, 0], PORT));
    let http_service = app.into_make_service_with_connect_info::<SocketAddr>();

    println!("Server started on port {PORT}");

    axum_server::bind(addr).serve(http_service).await
}
