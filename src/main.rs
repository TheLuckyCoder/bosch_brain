#![allow(dead_code)]

use env_logger::Env;
use tokio::task;
use crate::math::{Circle, Point};

use crate::serial::Message;
use crate::server::run_server_listeners;

mod math;
mod serial;
mod server;
#[cfg(test)]
mod tests;
mod track;
mod tui;

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        serial::send_blocking(Message::Speed(0_f32)).expect("Failed to force stop car");
        println!("Car stopped");
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // let _cleanup = Cleanup;

    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format_timestamp(None)
        .target(env_logger::Target::Stdout)
        .init();

    // let tui = task::spawn(tui::run());

    // let track = track::get_track();

    // let imu = sensors::get_imu().expect("Failed to initialize IMU");

    task::spawn(run_server_listeners());
    // tui.await??; // if the TUI task is finished, the program should exit


    let circle = Circle::find_center(Point::new(155.0,548.0),Point::new(126.0,546.0),Point::new(97.0,532.0));

    println!("Circle: {:?}", circle);

    Ok(())
}
