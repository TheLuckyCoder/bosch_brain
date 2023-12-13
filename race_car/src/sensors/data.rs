use std::time::{Duration, SystemTime};

use mint::{Quaternion, Vector3};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ImuData {
    pub quaternion: Quaternion<f32>,
    pub acceleration: Vector3<f32>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct GpsCoordinates {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub confidence: u8,
}

#[derive(Debug, Clone, Serialize)]
pub enum SensorData {
    Imu(ImuData),
    Distance(f32),
    Gps(GpsCoordinates),
}

#[derive(Debug, Clone, Serialize)]
pub struct TimedSensorData {
    #[serde(flatten)]
    pub data: SensorData,
    pub timestamp: Duration,
}

impl TimedSensorData {
    pub fn new(data: SensorData) -> Self {
        Self {
            data,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards"),
        }
    }
}
