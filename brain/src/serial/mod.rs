use std::mem::MaybeUninit;
#[cfg(test)]
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Once;
use std::time::Duration;
#[cfg(test)]
use tokio::task;

use tokio::task::JoinHandle;

use crate::serial::sender::{MessageSender, SerialMessageSender};

pub use self::message::*;

pub mod camera;
mod message;
mod sender;

fn init_serial() -> SerialMessageSender {
    let serial = SerialMessageSender(
        mio_serial::new("/dev/ttyACM0", 19200)
            .timeout(Duration::from_millis(100))
            .open()
            .expect("Failed to open serial port"),
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
pub fn get_test_queue() -> &'static mut (Sender<Message>, Receiver<Message>) {
    static mut SINGLETON: MaybeUninit<(Sender<Message>, Receiver<Message>)> = MaybeUninit::uninit();
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            SINGLETON.write(channel::<Message>());
        });

        SINGLETON.assume_init_mut()
    }
}

/**
 * Send a message to the nucleo board on the current thread.
 */
#[cfg(not(test))]
pub fn send_blocking(message: Message) -> std::io::Result<()> {
    get_serial().send_blocking(message)
}

/**
 * Send a message to the nucleo board asynchronously on another thread.
 */
#[cfg(not(test))]
pub fn send(message: Message) -> JoinHandle<()> {
    get_serial().send(message)
}

#[cfg(test)]
pub fn send_blocking(message: Message) -> std::io::Result<()> {
    let (sender, _) = get_test_queue();
    sender
        .clone()
        .send(message)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
}

#[cfg(test)]
pub fn send(message: Message) -> JoinHandle<()> {
    task::spawn_blocking(|| send_blocking(message).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_through_queue() {
        let (_, rx) = get_test_queue();

        let sent_message = Message::Speed(1.0);
        send_blocking(sent_message.clone()).unwrap();

        let received_message = rx.recv().unwrap();
        assert_eq!(sent_message, received_message);
    }
}
