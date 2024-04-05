use std::io::Read;
use std::time::Duration;

use tracing::{error, warn};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::http::GlobalState;
use crate::sensors::manager::SensorManager;
use crate::sensors::motor_driver::{Motor, MotorDriver};
use crate::sensors::set_board_led_status;

mod http;
mod sensors;
mod utils;
mod actuator;

/// Entrypoint of the program
///
/// Initializes the logging system, creates the GlobalState object and starts the HTTP server
#[tokio::main]
async fn main() -> Result<(), String> {
    std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().compact())
        .with(EnvFilter::from_default_env())
        .init();

    set_board_led_status(false).inspect_err(|e| error!("Failed to set board led: {e}")).ok();

    // let mut motor_driver = MotorDriver::new().unwrap();

    let sensor_manager = SensorManager::new();
    let global_state = GlobalState::new(sensor_manager);

    http::http_server(global_state).await.unwrap();

    return Ok(());
}
