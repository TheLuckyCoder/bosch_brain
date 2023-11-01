#![allow(dead_code)]

use std::sync::Arc;
use std::time::Duration;

use tokio::task;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use sensors::{DistanceSensor, GenericImu, MotorDriver, SensorManager};

use crate::http::GlobalState;

mod http;
mod math;
mod serial;
mod server;
#[cfg(test)]
mod tests;
mod track;
mod utils;

#[tokio::main]
async fn main() -> Result<(), String> {
    std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().compact())
        .with(EnvFilter::from_default_env())
        .init();

    println!("Start server? [y/N]");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    let mut motor_driver = MotorDriver::new().unwrap();

    if input.trim().to_ascii_lowercase() == "y" {
        let sensor_manager = Arc::new(SensorManager::new());
        let global_state = GlobalState::new(sensor_manager, motor_driver);

        task::spawn(http::http_server(global_state))
            .await
            .unwrap()
            .unwrap();

        return Ok(());
    }

    println!("Started manual mode");

    let mut imu = GenericImu::new().unwrap();
    let mut distance_sensor = DistanceSensor::new(22f32).unwrap();

    let mut angle = 0;
    loop {
        println!(
            "Acc: {:?} ; Quat: {:?}",
            imu.get_acceleration(),
            imu.get_quaternion()
        );
        println!(
            "Distance: {}",
            distance_sensor.get_distance_cm().unwrap_or(f32::NAN)
        );
        motor_driver.set_acceleration(0.5);
        motor_driver.set_steering_angle(angle as f64 / 10.0);
        angle += 1;
        if angle == 9 {
            angle = 0;
        }
        std::thread::sleep(Duration::from_millis(400));
    }
    // Ok(())
}
