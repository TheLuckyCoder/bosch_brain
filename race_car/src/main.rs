use std::sync::Arc;
use std::time::Duration;

use crate::http::GlobalState;
use crate::sensors::{DistanceSensor, GenericImu, Motor, MotorDriver, SensorManager};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

mod http;
mod sensors;

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

        http::http_server(global_state).await.unwrap();

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
        motor_driver.set_motor_value(Motor::Acceleration, 0.5);
        motor_driver.set_motor_value(Motor::Steering, angle as f64 / 10.0);
        angle += 1;
        if angle == 9 {
            angle = 0;
        }
        std::thread::sleep(Duration::from_millis(400));
    }
    // Ok(())
}
