#![allow(dead_code)]

use std::io::Read;
use std::sync::Arc;
use std::time::Duration;

use tokio::task;
use tracing::log::LevelFilter;
use tracing::Level;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use sensors::{DistanceSensor, GenericImu, MotorDriver, SensorManager};

use crate::http::GlobalState;
use crate::motor_manager::MotorManager;

mod http;
mod math;
mod motor_manager;
mod serial;
mod server;
#[cfg(test)]
mod tests;
mod track;
mod udp_manager;
mod utils;

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().compact())
        // .with(LevelFilter::Debug)
        .init();

    println!("Start server? [y/N]");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    if input.to_ascii_lowercase() == "y" {
        let sensor_manager = Arc::new(SensorManager::new());
        let motor_manager = Arc::new(MotorManager::new());
        let global_state = GlobalState::new(sensor_manager, motor_manager);

        task::spawn(http::http_server(global_state))
            .await
            .unwrap()
            .unwrap();
    }

    println!("Started manual mode");

    let mut imu = GenericImu::new().unwrap();
    let mut distance_sensor = DistanceSensor::new(22f32).unwrap();
    let mut motor_diver = MotorDriver::new().unwrap();

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
        motor_diver.set_acceleration(0.5);
        motor_diver.set_steering_angle(angle as f64 / 10.0);
        angle += 1;
        if angle == 9 {
            angle = 0;
        }
        std::thread::sleep(Duration::from_millis(400));
    }
    // Ok(())
}
