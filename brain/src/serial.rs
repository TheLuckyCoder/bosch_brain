use std::io::Read;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use serialport::{Parity, SerialPort, StopBits};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerialData {
    acceleration: [f32; 3],
    quaternion: [f32; 4],
    lateral_distance: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerialActions {
    acceleration: f64,
    steering: f64,
    camera_angle: f64,
}

pub struct SerialManager {
    pub port: Box<dyn SerialPort>,
    pub buffer: String,
}

impl SerialManager {
    pub fn new() -> Self {
        Self {
            port: mio_serial::new("/dev/ttyACM0", 19200)
                .timeout(Duration::from_millis(100))
                .stop_bits(StopBits::One)
                .open()
                .expect("Failed to open serial port"),
            buffer: String::with_capacity(512),
        }
    }

    pub fn read_data(&mut self) -> Option<SerialActions> {
        self.buffer.clear();
        match self.port.read_to_string(&mut self.buffer) {
            Ok(_) => {
                serde_json::from_str(&self.buffer).ok()
            }
            Err(e) => {
                eprintln!("Failed to read data: {e}");
                None
            }
        }
    }
}
