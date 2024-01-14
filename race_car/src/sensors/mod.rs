pub use data::*;
pub use gps::*;
pub use imu::*;
use linux_embedded_hal::gpio_cdev::{Chip, LineRequestFlags};
pub use manager::*;
pub use motor_driver::*;
use std::time::SystemTime;
pub use ambience::*;
pub use ultrasonic::*;

mod data;
mod gps;
mod imu;
mod manager;
mod motor_driver;
mod ambience;
mod ultrasonic;

pub trait BasicSensor {
    fn name(&self) -> &'static str;

    fn read_data(&mut self) -> SensorData;

    fn read_config(&mut self) -> String {
        self.read_data().to_string()
    }

    fn save_config(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn read_data_timed(&mut self, start_time: SystemTime) -> TimedSensorData {
        TimedSensorData::new(self.read_data(), start_time)
    }
}

pub fn set_board_led_status(on: bool) {
    let mut chip = Chip::new("/dev/gpiochip0").unwrap();
    let output = chip.get_line(25).unwrap();

    output
        .request(LineRequestFlags::OUTPUT, on as u8, "blinky")
        .unwrap();
}
