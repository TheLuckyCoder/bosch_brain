use crate::serial::Message;
use serialport::SerialPort;
use std::io::Write;
use std::net::TcpStream;
use std::str;
use tokio::task;
use tokio::task::JoinHandle;

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
            Ok(size) => log::debug!(
                "Response for \"{}\": {:?}",
                string.trim(),
                str::from_utf8(&result[..size])
            ),
            Err(e) => log::debug!("No response for \"{}\": {}", string.trim(), e),
        }

        Ok(())
    }

    fn send(&'static mut self, message: Message) -> JoinHandle<()> {
        task::spawn_blocking(|| self.send_blocking(message).unwrap())
    }
}

pub struct TcpMessageSender(pub TcpStream);

impl MessageSender for TcpMessageSender {
    fn send_blocking(&'static mut self, message: Message) -> std::io::Result<()> {
        self.0.write_all(message.to_string().as_bytes())
    }

    fn send(&'static mut self, message: Message) -> JoinHandle<()> {
        task::spawn_blocking(|| self.send_blocking(message).unwrap())
    }
}
