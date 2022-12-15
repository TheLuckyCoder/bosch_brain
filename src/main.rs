mod sensors;
mod serial;
mod server;

use crate::serial::Message;

#[tokio::main]
async fn main() {
    let port = serial::get_serial();

    match port.send_blocking(Message::speed(0.2_f32)) {
        Ok(message) => log::info!("Response: {message}"),
        Err(e) => log::error!("Error occurred while sending command: {e}"),
    }

    port.send(Message::speed(0.0_f32))
        .await
        .expect("It should have stopped");
}
