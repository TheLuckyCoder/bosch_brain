#![allow(dead_code)]

use env_logger::Env;
use sensors::distance::DistanceSensor;
use sensors::imu::GenericImu;
use sensors::motor_driver::MotorDriver;
use std::fs::OpenOptions;
use std::io::{BufRead, Read};
use std::net::TcpListener;
use std::path::Path;
use std::thread::sleep;
use tokio::task;

use crate::serial::Message;
use crate::state_machine::StateMachine;

mod brain;
mod math;
mod serial;
mod server;
mod state_machine;
#[cfg(test)]
mod tests;
mod track;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format_timestamp(None)
        .target(env_logger::Target::Stdout)
        .init();

    let mut imu = GenericImu::new().unwrap();
    let mut distance_sensor = DistanceSensor::new(imu.get_temperature().unwrap() as f32);
    let mut motor_diver = MotorDriver::new().unwrap();

    let listener = TcpListener::bind("192.168.0.1:12345")?;
    let mut state_machine = StateMachine::new();

    let mut buffer = String::with_capacity(128);
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                stream.read_to_string(&mut buffer)?;
                match buffer.as_str() {
                    "State:S" => state_machine.to_standby(),
                    "State:A" => state_machine.to_autonomous_controlled(),
                    "State:R" => state_machine.to_remote_controlled(),
                    _ => log::error!("Invalid message: {buffer}"),
                };
            }
            Err(err) => log::error!("{err}"),
        }
    }

    // let track = track::get_track();

    // let path = "/home/car/recorded_movements/full_run.txt";

    // task::spawn(async move {
    //     if let Err(e) = server::steering_wheel::run_steering_wheel_server(path).await {
    //         log::error!("Steering wheel server error: {e}");
    //     }
    // });
    // if Path::new(path).exists() {
    //     let file = OpenOptions::new().read(true).open(path)?;
    //
    //     // read all lines from file and store them in a Vec
    //     let lines: Vec<_> = std::io::BufReader::new(file)
    //         .lines()
    //         .map(|l| l.unwrap())
    //         .collect();
    //
    //     for line in lines {
    //         let mut split = line.split('|');
    //         let time = split.next().unwrap();
    //         let message = split.next().unwrap();
    //
    //         serial::send_blocking(Message::Raw(message.to_string()))?;
    //         //sleep for time milliseconds
    //         sleep(std::time::Duration::from_millis(
    //             time.parse::<u64>().unwrap(),
    //         ));
    //     }
    // }
    // serial::send_blocking(Message::Speed(0_f32))?; //stop car
    // brain::start_brain();

    Ok(())
}
