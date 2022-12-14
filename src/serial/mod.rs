use std::io::{Read, Write};
use std::mem::MaybeUninit;
use std::sync::Once;

use tokio_serial::SerialStream;

pub use self::message::*;

mod message;

pub struct MainSerialPort(SerialStream);

impl MainSerialPort {
    pub fn send_blocking(&mut self, message: Message) -> std::io::Result<String> {
        let mut result = String::with_capacity(128);
        self.0.write_all(message.get_bytes())?;
        self.0.read_to_string(&mut result)?;
        Ok(result)
    }
}

fn init_serial() -> MainSerialPort {
    MainSerialPort(
        SerialStream::open(&tokio_serial::new("/dev/ttyACM0", 19200)).expect("Failed to open port"),
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
