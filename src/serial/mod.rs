use serialport::SerialPort;
use std::io::{Read, Write};
use std::mem::MaybeUninit;
use std::str;
use std::sync::Once;
use tokio::task;
use tokio::task::JoinHandle;

pub use self::message::*;

mod message;

pub struct SerialManager(Box<dyn SerialPort>);

impl SerialManager {
    /**
     * Send a message to the nucleo board on the current thread.
     */
    pub fn send_blocking(&mut self, message: Message) -> std::io::Result<()> {
        self.0.write_all(message.get_bytes())?;
        let mut result = [0_u8; 512];

        match self.0.read(&mut result) {
            Ok(size) => log::debug!(
                "Response for \"{}\": {:?}",
                message.get_string().trim(),
                str::from_utf8(&result[..size])
            ),
            Err(e) => log::debug!("No response for \"{}\": {}", message.get_string().trim(), e),
        }

        Ok(())
    }

    /**
     * Send a message to the nucleo board asynchronously on another thread.
     */
    pub fn send(&'static mut self, message: Message) -> JoinHandle<()> {
        task::spawn_blocking(|| self.send_blocking(message).unwrap())
    }
}

fn init_serial() -> SerialManager {
    let serial = SerialManager(
        mio_serial::new("/dev/ttyACM0", 19200)
            .open()
            .expect("Failed to open port"),
    );

    log::info!("Serial port initialized");

    serial
}

pub fn get_serial() -> &'static mut SerialManager {
    static mut SINGLETON: MaybeUninit<SerialManager> = MaybeUninit::uninit();
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            SINGLETON.write(init_serial());
        });

        SINGLETON.assume_init_mut()
    }
}
