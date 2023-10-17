#![allow(dead_code)]

use std::io::{BufRead, Read, Write};
use std::time::Duration;

use env_logger::Env;
use sensors::{DistanceSensor, GenericImu, MotorDriver};

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
    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format_timestamp(None)
        .target(env_logger::Target::Stdout)
        .init();

    let mut imu = GenericImu::new().unwrap();
    let mut distance_sensor = DistanceSensor::new(22f32).unwrap();
    let mut motor_diver = MotorDriver::new().unwrap();

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
        motor_diver.set_acceleration(0.2);
        motor_diver.set_steering_angle(1.0);
        std::thread::sleep(Duration::from_millis(200));
    }
    Ok(())
}
