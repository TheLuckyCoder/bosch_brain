#![allow(dead_code)]

use std::env;

use env_logger::Env;
use tokio::task;

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
    env::set_var("RUST_BACKTRACE", "full");
    // let _cleanup = Cleanup;

    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format_timestamp(None)
        .target(env_logger::Target::Stdout)
        .init();

    // let tui = task::spawn(tui::run());

    let track = track::get_track();

    // match imu::get_imu() {
    //     Ok(mut imu) => {
    //         log::info!("Gyro: {:?}", imu.get_gyro().unwrap());
    //     }
    //     Err(e) => log::error!("Failed to initialize IMU: {}", e),
    // }

    task::spawn(run_server_listeners());
    // tui.await??; // if the TUI task is finished, the program should exit

    let start_node = match track.get_node_by_id(24) {
        Some(node) => node,
        None => panic!("start node not found"),
    };

    let end_node = match track.get_node_by_id(22) {
        Some(node) => node,
        None => panic!("end node not found"),
    };

    let path = track::find_path(track, start_node, end_node).unwrap();

    log::debug!("Start node: {:?}\n", start_node);
    log::info!("Path length: {}\n", path.len());
    for node in path {
        log::info!("Node: {:?}", node);
    }
    log::info!("End node: {:?}\n", end_node);

    Ok(())
}
