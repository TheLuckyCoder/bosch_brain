pub use ambience::*;
use anyhow::Context;
pub use data::*;
pub use gps::*;
pub use imu::*;
use linux_embedded_hal::gpio_cdev::{Chip, LineRequestFlags};
use std::time::SystemTime;
pub use ultrasonic::*;

mod ambience;
mod data;
mod gps;
mod imu;
pub mod manager;
pub mod motor_driver;
mod ultrasonic;

/// Common set of functions each sensor class should implement
pub trait BasicSensor {
    /// Unique name of the sensor
    fn name(&self) -> &'static str;

    /// Blocking function that reads the data from the sensor
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

/// Helper function to set the board LED status
pub fn set_board_led_status(on: bool) -> anyhow::Result<()> {
    let mut chip = Chip::new("/dev/gpiochip0").context("Failed to open GPIO file")?;
    let output = chip.get_line(25).context("Failed to get GPIO PIN 25")?;

    output
        .request(LineRequestFlags::OUTPUT, on as u8, "blinky")
        .map(|_| ())
        .context("Failed to set GPIO PIN 25")
}
