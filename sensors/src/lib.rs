use crate::name::SensorName;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub mod drivers;
pub mod name;

/// Common set of functions each sensor class should implement
pub trait HardwareSensor: Send + 'static {
    /// Unique name of the sensor
    fn name(&self) -> SensorName;

    /// Called right before a reading session begins
    fn prepare_read(&mut self) {}

    /// Reads data from the sensor, returning a generic [SensorData] enum
    fn read_data(&mut self) -> SensorData;

    fn start_calibration(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Allows the sensor to read its debug data, needed for configuration, defaults to [Self::read_data]
    fn read_debug(&mut self) -> String {
        self.read_data().to_string()
    }

    /// Allows the sensor to save its current configuration, defaults to doing nothing
    fn end_calibration(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GpsCoordinates {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub confidence: u8,
}

/// Enum containing all possible sensor data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SensorData {
    Imu {
        quaternion: [f32; 4],
        acceleration: [f32; 3],
    },
    Bno085 {
        euler_angles: [f32; 3],
        acceleration: [f32; 3],
    },
    Ultrasonic(f32),
    Gps(GpsCoordinates),
    MotorRPS(f64),
    VirtualVelocity {
        imu: f64,
        rot_encoder: f64,
        fusion: f64,
    },
}

impl Display for SensorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensorData::Imu {
                quaternion,
                acceleration,
            } => write!(
                f,
                "Quaternion: {:.4?}, Acceleration: {:.4?}",
                quaternion, acceleration
            ),
            SensorData::Bno085 {
                euler_angles,
                acceleration,
            } => write!(
                f,
                "Euler Angles: {:.4?}, Acceleration: {:.4?}",
                euler_angles, acceleration
            ),
            SensorData::Ultrasonic(distance) => write!(f, "Ultrasonic: {distance:.4}"),
            SensorData::Gps(coordinates) => write!(f, "{coordinates:.4?}"),
            SensorData::MotorRPS(rps) => write!(f, "Shaft RPS: {rps:.4?} | RPM: {:.4?}", rps * 60.0),
            SensorData::VirtualVelocity { imu, rot_encoder, fusion } => {
                write!(
                    f,
                    "Virtual Velocity - IMU: {:.4?} m/s, Rotary Encoder: {:.4?} m/s, Fused: {:.4?} m/s",
                    imu, rot_encoder, fusion
                )
            }
        }
    }
}

impl SensorData {
    pub fn get_sensor_name(&self) -> SensorName {
        match self {
            SensorData::Imu { .. } => SensorName::Imu,
            SensorData::Bno085 { .. } => SensorName::Bno085,
            SensorData::Ultrasonic(_) => SensorName::Ultrasonic,
            SensorData::Gps(_) => SensorName::Gps,
            SensorData::MotorRPS(_) => SensorName::OpticalRotaryEncoder,
            SensorData::VirtualVelocity { .. } => SensorName::VirtualVelocity,
        }
    }
}
