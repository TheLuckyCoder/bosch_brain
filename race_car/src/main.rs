use anyhow::Context;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use ::sensors::drivers::set_board_led_status;
use crate::actuators::{add_all_actuators, add_all_mock_actuators};

use crate::actuators::manager::ActuatorManager;
use crate::http::config::ServerConfig;
use crate::http::GlobalState;
use crate::sensors::add_all_sensors;
use crate::sensors::manager::SensorManager;
use crate::sensors::mock::add_all_mock_sensors;

mod actuators;
mod http;
mod sensors;
mod utils;

/// Entrypoint of the program
///
/// Initializes the logging system, creates the GlobalState object and starts the HTTP server
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().compact())
        .with(EnvFilter::from_default_env())
        .init();

    set_board_led_status(false)
        .inspect_err(|e| error!("Failed to set board led: {e}"))
        .ok();

    let server_config = ServerConfig::read_server_config().unwrap_or_else(|e| {
        error!("Failed to load config.toml: {e}");
        ServerConfig::default()
    });

    let mut sensor_manager = SensorManager::new();
    let mut actuator_manager = ActuatorManager::new();

    if server_config.mock_sensors {
        info!("Initializing with Mock Sensors");
        add_all_mock_sensors(&mut sensor_manager);
        add_all_mock_actuators(&mut actuator_manager);
    } else {
        add_all_sensors(&mut sensor_manager);
        add_all_actuators(&mut actuator_manager);
    }

    let global_state = GlobalState::new(sensor_manager, actuator_manager, server_config);

    http::http_server(global_state)
        .await
        .context("Failed to start HTTP Server")
}
