use std::io::Read;
use std::mem::{size_of, transmute};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use serialport::SerialPort;

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct LanesAngle {
    left: f64,
    right: f64,
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum CameraData {
    LanesAngle(LanesAngle) = 0,
}

fn parse_camera_data(serial: &mut CameraSerialReceiver) -> CameraData {
    let mut buffer = [0_u8; 512];
    serial.read_exact(buffer.as_mut()).unwrap();

    let discriminant = buffer[0];

    match discriminant {
        0 => {
            const SIZE: usize = size_of::<LanesAngle>();
            CameraData::LanesAngle(unsafe {
                transmute::<[u8; SIZE], LanesAngle>(buffer[1..SIZE].try_into().unwrap())
            })
        }
        _ => panic!("Unknown data type: {discriminant}"),
    }
}

type CameraSerialReceiver = Box<dyn SerialPort>;

fn get_camera_serial() -> CameraSerialReceiver {
    // TODO
    let serial = mio_serial::new("/dev/ttyACM1", 19200)
        .timeout(Duration::from_millis(1000))
        .open()
        .expect("Failed to open camera serial port");

    log::info!("Camera serial port initialized!");

    serial
}

pub fn get_camera_data_receiver() -> Receiver<CameraData> {
    let mut serial = get_camera_serial();
    let (sender, receiver) = std::sync::mpsc::channel();

    std::thread::spawn(move || loop {
        let data = parse_camera_data(&mut serial);
        sender.send(data).unwrap();
    });

    receiver
}
