mod sensors;
mod serial;
mod server;
mod tui;

use crate::sensors::imu::Imu;
use crate::serial::Message;
use crate::server::run_server_listeners;
use tokio::task;

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        let serial = serial::get_serial();
        serial.send_blocking(Message::speed(0_f32)).unwrap();
        println!("Car stopped forcefully");
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let _cleanup = Cleanup;
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    let tui = task::spawn(async move { tui::run().await });
    let port = serial::get_serial();

    port.send_blocking(Message::speed(0.2_f32))?;
    std::thread::sleep(std::time::Duration::from_secs(2));
    port.send(Message::speed(0.0_f32)).await?;

    match Imu::new() {
        Ok(mut imu) => {
            log::info!("Accel: {:?}", imu.0.accel_data().unwrap());
        }
        Err(e) => {
            log::error!("Failed to initialize IMU: {}", e);
        }
    }

    task::spawn(async move { run_server_listeners().await });
    tui.await??; // if the TUI task is finished, the program should exit
    Ok(())
}
