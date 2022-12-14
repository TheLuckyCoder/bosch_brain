mod serial;

use crate::serial::Message;

#[tokio::main]
async fn main() {
    let port = serial::get_serial();

    match port.send_blocking(Message::speed(0.2_f32)) {
        Ok(message) => println!("{message}"),
        Err(e) => println!("Error occurred while sending command: {e}"),
    }
}
