use std::io::Read;
use std::time::Duration;

use tracing::warn;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::http::GlobalState;
use crate::sensors::{set_board_led_status};
use crate::sensors::manager::SensorManager;
use crate::sensors::motor_driver::{Motor, MotorDriver};

mod http;
mod sensors;
mod utils;

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

    set_board_led_status(false).unwrap();

    let mut motor_driver = MotorDriver::new().unwrap();

    if false {
        let mut serial = serialport::new(
            "/dev/serial/by-id/usb-Arduino__www.arduino.cc__0043_55838343633351116232-if00",
            9600,
        )
        .open_native()
        .unwrap();

        let mut i = 0_usize;

        loop {
            let mut buffer = vec![0; 4096];

            match serial.read(buffer.as_mut_slice()) {
                Ok(bytes_read) => match String::from_utf8(buffer[..bytes_read].to_vec()) {
                    Ok(text) => println!("{}", text),
                    Err(e) => warn!("{e}"),
                },
                Err(_) => {
                    // error!("{e}")
                }
            }

            std::thread::sleep(Duration::from_millis(5));
            i += 1;

            if i % 10 == 0 {
                motor_driver.set_motor_value(Motor::Steering, 0.0);
            }
        }
    }

    // let mut sensor = AmbienceSensor::new().unwrap();
    // loop {
    //     println!("{}", sensor.read_data());
    // }

    let sensor_manager = SensorManager::new();
    let global_state = GlobalState::new(sensor_manager, motor_driver);

    http::http_server(global_state).await.unwrap();

    return Ok(());
}
