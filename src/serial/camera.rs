use std::io::Read;
use std::mem::{size_of, transmute};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use serialport::SerialPort;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct LanesAngle {
    pub left: f64,
    pub right: f64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Signs {
    pub stop: f64,
    pub crosswalk: f64,
    pub parking_start: f64,
    pub parking_stop: f64,
    pub priority: f64,
}

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum CameraData {
    LanesAngle(LanesAngle) = 0,
    Signs(Signs) = 1,
}

type CameraSerialReceiver = Box<dyn SerialPort>;

fn parse_camera_data(buffer: &[u8; 512]) -> CameraData {
    let discriminant = buffer[0];

    match discriminant {
        0 => {
            const SIZE: usize = size_of::<LanesAngle>();
            CameraData::LanesAngle(unsafe {
                transmute::<[u8; SIZE], LanesAngle>(buffer[1..SIZE + 1].try_into().unwrap())
            })
        }
        _ => panic!("Unknown data type: {discriminant}"),
    }
}

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
        let mut buffer = [0_u8; 512];
        serial.read_exact(buffer.as_mut()).unwrap();

        let data = parse_camera_data(&buffer);
        sender.send(data).unwrap();
    });

    receiver
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_camera_data() {
        let mut buffer = [0_u8; 512];
        const SIZE: usize = size_of::<LanesAngle>();
        let data = LanesAngle {
            left: 1.5,
            right: 2.0,
        };

        unsafe {
            let bytes: [u8; SIZE] = transmute(data);
            bytes.as_ref().iter().enumerate().for_each(|(i, v)| {
                buffer[i + 1] = *v;
            });
        }

        let parsed_data = parse_camera_data(&buffer);
        dbg!(data);
        assert_eq!(CameraData::LanesAngle(data), parsed_data);
    }
}
