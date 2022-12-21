use crate::sensors::imu;
use env_logger::Env;
use tokio::task;

use crate::serial::Message;
use crate::server::run_server_listeners;

mod sensors;
mod serial;
mod server;
mod track;
mod tui;

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        serial::get_serial()
            .send_blocking(Message::speed(0_f32))
            .expect("Failed to force stop car");
        println!("Car stopped");
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let _cleanup = Cleanup;

    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format_timestamp(None)
        .target(env_logger::Target::Stdout)
        .init();

    let tui = task::spawn(tui::run());
    let port = serial::get_serial();

    let track = track::get_track();
    println!("{}", track.0.len());

    port.send_blocking(Message::speed(0.1_f32))?;
    std::thread::sleep(std::time::Duration::from_secs(2));
    port.send(Message::speed(0.0_f32)).await?;

    match imu::get_imu() {
        Ok(mut imu) => {
            log::info!("Gyro: {:?}", imu.get_gyro().unwrap());
        }
        Err(e) => log::error!("Failed to initialize IMU: {}", e),
    }

    task::spawn(run_server_listeners());
    tui.await??; // if the TUI task is finished, the program should exit
    Ok(())
}
