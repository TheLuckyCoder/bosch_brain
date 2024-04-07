//! Module containing all sensor abstraction classes

use anyhow::Context;
use linux_embedded_hal::gpio_cdev::{Chip, LineRequestFlags};
use serde::Serialize;
use serde_with::TimestampMilliSeconds;
use serde_with::{serde_as, DeserializeFromStr, SerializeDisplay};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::SystemTime;
use strum::{AsRefStr, EnumIter, IntoStaticStr};

pub use ambience::*;
pub use gps::*;
pub use imu::*;
pub use ultrasonic::*;
pub use velocity::*;
pub use mock::*;

mod ambience;
mod gps;
mod imu;
pub mod manager;
pub mod motor_driver;
mod ultrasonic;
mod velocity;
mod mock;

/// Common set of functions each sensor class should implement
pub trait BasicSensor: Send + 'static {
    /// Unique name of the sensor
    fn name(&self) -> SensorName;

    /// Called right before a reading session begins
    fn prepare_read(&mut self) {}

    /// Reads data from the sensor, returning a generic [SensorData] enum
    fn read_data(&mut self) -> SensorData;

    /// Allows the sensor to read its debug data, needed for configuration, defaults to [Self::read_data]
    fn read_debug(&mut self) -> String {
        self.read_data().to_string()
    }

    /// Allows the sensor to save its current configuration, defaults to doing nothing
    fn save_config(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Reads data, and returns it with a timestamp
    fn read_data_timed(&mut self) -> TimedSensorData {
        TimedSensorData::from(self.read_data())
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

#[derive(
    Debug,
    Clone,
    Copy,
    DeserializeFromStr,
    SerializeDisplay,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    EnumIter,
    IntoStaticStr,
    AsRefStr,
)]
pub enum SensorName {
    Ambience,
    Gps,
    Imu,
    Ultrasonic,
    Velocity,
}

impl FromStr for SensorName {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "imu" => Ok(SensorName::Imu),
            "ultrasonic" => Ok(SensorName::Ultrasonic),
            "gps" => Ok(SensorName::Gps),
            "velocity" => Ok(SensorName::Velocity),
            "ambience" => Ok(SensorName::Ambience),
            _ => Err("No such Sensor exists"),
        }
    }
}

impl Display for SensorName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.into())
    }
}

/// Enum containing all possible sensor data
#[derive(Debug, Clone, Serialize)]
pub enum SensorData {
    Imu(ImuData),
    Ultrasonic(f32),
    Gps(GpsCoordinates),
    Velocity(f64),
    Ambience(AmbienceData),
}

impl Display for SensorData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SensorData::Imu(imu_data) => write!(f, "Imu: {imu_data}"),
            SensorData::Ultrasonic(distance) => write!(f, "Ultrasonic: {distance:.4}"),
            SensorData::Gps(coordinates) => write!(f, "{coordinates:?}"),
            SensorData::Velocity(velocity) => write!(f, "Velocity: {velocity:.4}"),
            SensorData::Ambience(ambience) => write!(f, "Ambience: {ambience:?}"),
        }
    }
}

impl SensorData {
    pub fn get_sensor_name(&self) -> SensorName {
        match self {
            SensorData::Imu(_) => SensorName::Imu,
            SensorData::Ultrasonic(_) => SensorName::Ultrasonic,
            SensorData::Gps(_) => SensorName::Gps,
            SensorData::Velocity(_) => SensorName::Velocity,
            SensorData::Ambience(_) => SensorName::Ambience,
        }
    }
}

/// Sensor data with a timestamp
#[serde_as]
#[derive(Debug, Clone, Serialize)]
pub struct TimedSensorData {
    #[serde(flatten)]
    pub data: SensorData,
    #[serde_as(as = "TimestampMilliSeconds<i64>")]
    #[serde(rename = "timestamp_ms")]
    pub timestamp: SystemTime,
}

impl TimedSensorData {
    pub fn new(data: SensorData, timestamp: SystemTime) -> Self {
        Self { data, timestamp }
    }
}

impl From<SensorData> for TimedSensorData {
    fn from(value: SensorData) -> Self {
        Self::new(value, SystemTime::now())
    }
}
