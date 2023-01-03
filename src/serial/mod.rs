use crate::serial::sender::{MessageSender, SerialMessageSender, TcpMessageSender};
use std::mem::MaybeUninit;
use std::net::TcpStream;
use std::sync::Once;
use tokio::task::JoinHandle;

pub use self::message::*;

mod message;
mod sender;

fn init_serial() -> SerialMessageSender {
    let serial = SerialMessageSender(
        mio_serial::new("/dev/ttyACM0", 19200)
            .open()
            .expect("Failed to open port"),
    );

    log::info!("Serial port initialized");

    serial
}

fn get_serial() -> &'static mut SerialMessageSender {
    static mut SINGLETON: MaybeUninit<SerialMessageSender> = MaybeUninit::uninit();
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            SINGLETON.write(init_serial());
        });

        SINGLETON.assume_init_mut()
    }
}

#[cfg(test)]
pub fn get_tcp() -> &'static mut TcpMessageSender {
    static mut SINGLETON: MaybeUninit<TcpMessageSender> = MaybeUninit::uninit();
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            SINGLETON.write(TcpMessageSender(
                TcpStream::connect("0.0.0.0:25565").unwrap(),
            ));
        });

        SINGLETON.assume_init_mut()
    }
}

/**
 * Send a message to the nucleo board on the current thread.
 */
pub fn send_blocking(message: Message) -> std::io::Result<()> {
    get_serial().send_blocking(message)
}

/**
 * Send a message to the nucleo board asynchronously on another thread.
 */
pub fn send(message: Message) -> JoinHandle<()> {
    get_serial().send(message)
}
