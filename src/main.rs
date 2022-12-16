use env_logger::Env;
use tokio::task;

use crate::sensors::imu::Imu;
use crate::serial::Message;
use crate::server::run_server_listeners;

mod sensors;
mod serial;
mod server;
mod tui;

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        let serial = serial::get_serial();
        serial
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
    let tui = task::spawn(async move { tui::run().await });
    let port = serial::get_serial();

    port.send_blocking(Message::speed(0.1_f32))?;
    std::thread::sleep(std::time::Duration::from_secs(2));
    port.send(Message::speed(0.0_f32)).await?;

    match Imu::new() {
        Ok(mut imu) => {
            log::info!("Gyro: {:?}", imu.0.get_gyro().unwrap());
        }
        Err(e) => {
            log::error!("Failed to initialize IMU: {}", e);
        }
    }

    task::spawn(async move { run_server_listeners().await });
    tui.await??; // if the TUI task is finished, the program should exit
    Ok(())
}
