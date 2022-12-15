mod sensors;
mod serial;
mod server;

use crate::serial::Message;
use crate::server::run_server_listeners;
use std::thread::sleep;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    let port = serial::get_serial();

    sleep(std::time::Duration::from_secs(2));
    port.send_blocking(Message::speed(0.2_f32))?;
    sleep(std::time::Duration::from_secs(1));

    port.send(Message::speed(0.0_f32)).await?;

    run_server_listeners().await;
    Ok(())
}
