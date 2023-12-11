use mint::{Quaternion, Vector3};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct GpsCoordinates {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub confidence: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SensorData {
    Imu {
        quaternion: Quaternion<f32>,
        acceleration: Vector3<f32>,
    },
    Distance(f32),
    Gps(GpsCoordinates),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimedSensorData {
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
