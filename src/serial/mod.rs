use std::io::{Read, Write};
use std::mem::MaybeUninit;
use std::sync::Once;

use serialport::TTYPort;
use tokio::task;
use tokio::task::JoinHandle;

pub use self::message::*;

mod message;

pub struct MainSerialPort(TTYPort);

impl MainSerialPort {
    pub fn send_blocking(&mut self, message: Message) -> std::io::Result<()> {
        let mut result = String::with_capacity(128);
        self.0.write_all(message.get_bytes())?;

        match self.0.read_to_string(&mut result) {
            Ok(_) => log::debug!(
                "Response for \"{}\": {}",
                message.get_string().trim(),
                result
            ),
            Err(e) => log::debug!("No response for \"{}\": {}", message.get_string().trim(), e),
        }

        Ok(())
    }

    pub fn send(&'static mut self, message: Message) -> JoinHandle<()> {
        task::spawn_blocking(|| self.send_blocking(message).unwrap())
    }
}

fn init_serial() -> MainSerialPort {
    MainSerialPort(
        mio_serial::new("/dev/ttyACM0", 19200)
            .open_native()
            .expect("Failed to open port"),
    )
}

pub fn get_serial() -> &'static mut MainSerialPort {
    static mut SINGLETON: MaybeUninit<MainSerialPort> = MaybeUninit::uninit();
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            SINGLETON.write(init_serial());
        });

        SINGLETON.assume_init_mut()
    }
}
