use serde::Serialize;
use std::io::{Read, Write};
use std::time::Duration;

use serialport::{DataBits, Parity, SerialPort, StopBits, TTYPort};
use tracing::{error, info};

use crate::sensors::{BasicSensor, SensorData, SensorName};

/// Data from the GPS sensor
#[derive(Debug, Clone, Copy, Serialize)]
pub struct GpsCoordinates {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub confidence: u8,
}

/// GPS Sensor, wrapper for the J-Link
pub struct GpsSensor {
    serial: TTYPort,
    buffer: Vec<u8>,
    initialized: bool,
}

impl GpsSensor {
    pub fn new() -> anyhow::Result<GpsSensor> {
        let serial = serialport::new(
            "/dev/serial/by-id/usb-SEGGER_J-Link_000760170010-if00",
            115200,
        )
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .timeout(Duration::from_millis(200))
        .open_native()?;

        Ok(Self {
            serial,
            buffer: vec![0; 4096],
            initialized: false,
        })
    }

    pub fn get_coordinates(&mut self) -> GpsCoordinates {
        if !self.initialized {
            self.init();
        }

        loop {
            let line = match self.read() {
                Some(value) => value,
                None => continue,
            };

            let values = match line
                .rsplit_once(" est[")
                .and_then(|(_, string)| string.split_once(']'))
            {
                Some((v, _)) => v,
                None => continue,
            };

            let coordinates: Vec<_> = values
                .split(',')
                .filter_map(|v| v.parse::<f32>().ok())
                .collect();

            return GpsCoordinates {
                x: coordinates[0],
                y: coordinates[1],
                z: coordinates[2],
                confidence: coordinates[3] as u8,
            };
        }
    }

    fn init(&mut self) {
        self.serial.write_all(b"\r").unwrap();
        self.serial.flush().unwrap();
        self.serial.write_all(b"\r").unwrap();
        self.serial.flush().unwrap();
        std::thread::sleep(Duration::from_secs(1));
        while let Err(e) = self.serial.write_all(b"les\r") {
            error!("{e}");
            std::thread::sleep(Duration::from_secs(1));
        }

        self.initialized = true;
    }

    fn read(&mut self) -> Option<String> {
        let bytes_to_read = self.serial.bytes_to_read().ok()?;
        if bytes_to_read < 137 {
            // info!("Only has {bytes_to_read} bytes");
            std::thread::sleep(Duration::from_millis(10)); // Tested to be stable, and has enough precision
            return None;
        }

        match self.serial.read(self.buffer.as_mut_slice()) {
            Ok(bytes_read) => {
                // info!(
                //     "String read: {}",
                //     String::from_utf8(self.buffer[..bytes_read].to_vec()).unwrap()
                // );
                if bytes_read == 0 {
                    self.serial.write_all(b"\r\r").unwrap();
                    None
                } else {
                    String::from_utf8(self.buffer[..bytes_read].to_vec()).ok()
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                if let Err(err) = self.serial.write_all(b"\r\r") {
                    error!("Write error: {err}");
                }
                None
            }
            Err(e) => {
                error!("Read error: {e}");
                None
            }
        }
    }
}

impl BasicSensor for GpsSensor {
    fn name(&self) -> SensorName {
        SensorName::Gps
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Gps(self.get_coordinates())
    }
}
