//! Module containing all sensor abstraction classes

use std::fmt::{Display, Formatter};
use std::time::SystemTime;

use serde::Serialize;
use serde_with::serde_as;
use serde_with::TimestampMilliSeconds;

pub use name::*;

pub mod drivers;
pub mod manager;
pub mod mock;
mod motor_driver;
mod name;

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

#[derive(Debug, Clone, Copy, Serialize)]
pub struct GpsCoordinates {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub confidence: u8,
}

/// Enum containing all possible sensor data
#[derive(Debug, Clone, Serialize)]
pub enum SensorData {
    Imu {
        quaternion: [f32; 4],
        acceleration: [f32; 3],
    },
    Ultrasonic(f32),
    Gps(GpsCoordinates),
    Velocity(f64),
    Ambience {
        temperature: f32,
        humidity: f32,
    },
}

impl Display for SensorData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SensorData::Imu {
                quaternion,
                acceleration,
            } => write!(
                f,
                "Quaternion: {:?}, Acceleration: {:?}",
                quaternion, acceleration
            ),
            SensorData::Ultrasonic(distance) => write!(f, "Ultrasonic: {distance:.4}"),
            SensorData::Gps(coordinates) => write!(f, "{coordinates:?}"),
            SensorData::Velocity(velocity) => write!(f, "Velocity: {velocity:.4}"),
            SensorData::Ambience {
                temperature,
                humidity,
            } => write!(f, "Ambience: {temperature:.4}, {humidity:.4}"),
        }
    }
}

impl SensorData {
    pub fn get_sensor_name(&self) -> SensorName {
        match self {
            SensorData::Imu { .. } => SensorName::Imu,
            SensorData::Ultrasonic(_) => SensorName::Ultrasonic,
            SensorData::Gps(_) => SensorName::Gps,
            SensorData::Velocity(_) => SensorName::Velocity,
            SensorData::Ambience { .. } => SensorName::Ambience,
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
