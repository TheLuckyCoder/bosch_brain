use std::io::Write;
use std::str;

use serialport::SerialPort;
use tokio::task;
use tokio::task::JoinHandle;
use tracing::debug;

use crate::serial_old::Message;

pub trait MessageSender {
    fn send_blocking(&'static mut self, message: Message) -> std::io::Result<()>;

    /**
     * Send a message to the asynchronously
     */
    fn send(&'static mut self, message: Message) -> JoinHandle<()>;
}

pub struct SerialMessageSender(pub Box<dyn SerialPort>);

impl MessageSender for SerialMessageSender {
    fn send_blocking(&'static mut self, message: Message) -> std::io::Result<()> {
        let string = message.to_string();
        self.0.write_all(string.as_bytes())?;
        let mut result = [0_u8; 512];

        match self.0.read(&mut result) {
            Ok(size) => debug!(
                "Response for \"{}\": {:?}",
                string.trim(),
                str::from_utf8(&result[..size])
            ),
            Err(e) => debug!("No response for \"{}\": {}", string.trim(), e),
        }

        Ok(())
    }

    fn send(&'static mut self, message: Message) -> JoinHandle<()> {
        task::spawn_blocking(|| self.send_blocking(message).unwrap())
    }
}
