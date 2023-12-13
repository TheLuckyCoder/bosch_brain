use std::io::{Read, Write};
use std::time::Duration;

use serialport::{DataBits, Parity, StopBits, TTYPort};
use tracing::{error, info};

use crate::sensors::{BasicSensor, GpsCoordinates, SensorData};

pub struct Gps {
    serial: TTYPort,
    buffer: Vec<u8>,
}

impl Gps {
    pub const NAME: &'static str = "GPS";

    pub fn new() -> anyhow::Result<Gps> {
        let mut serial = serialport::new(
            "/dev/serial/by-id/usb-SEGGER_J-Link_000760170010-if00",
            115200,
        )
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .timeout(Duration::from_millis(200))
        .open_native()?;

        info!("Serial initialized");

        while let Err(e) = serial.write_all(b"\n\n") {
            error!("{e}");
            std::thread::sleep(Duration::from_secs(1));
        }
        while let Err(e) = serial.write_all(b"les") {
            error!("{e}");
            std::thread::sleep(Duration::from_secs(1));
        }

        Ok(Self {
            serial,
            buffer: vec![0; 4096],
        })
    }

    pub fn get_coordinates(&mut self) -> GpsCoordinates {
        loop {
            let line = match self.read() {
                Some(value) => value,
                None => continue,
            };

            info!("GPS: {line}");
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
                confidence: coordinates[4] as u8,
            };
        }
    }

    fn read(&mut self) -> Option<String> {
        match self.serial.read(self.buffer.as_mut_slice()) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    self.serial.write_all(b"\n").unwrap();
                    None
                } else {
                    String::from_utf8(self.buffer[..bytes_read].to_vec()).ok()
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                if let Err(err) = self.serial.write_all(b"\n") {
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

impl BasicSensor for Gps {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Gps(self.get_coordinates())
    }
}
