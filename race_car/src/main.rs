use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::http::GlobalState;
use crate::sensors::{MotorDriver, SensorManager};

mod files;
mod http;
mod sensors;

#[tokio::main]
async fn main() -> Result<(), String> {
    std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().compact())
        .with(EnvFilter::from_default_env())
        .init();

    let motor_driver = MotorDriver::new().unwrap();

    let sensor_manager = SensorManager::new();
    let global_state = GlobalState::new(sensor_manager, motor_driver);

    http::http_server(global_state).await.unwrap();

    return Ok(());
}
