use std::time::{Duration, SystemTime};

use mint::{Quaternion, Vector3};
use serde::Serialize;
use serde_with::serde_as;
use serde_with::DurationMilliSeconds;

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

#[serde_as]
#[derive(Debug, Clone, Serialize)]
pub struct TimedSensorData {
    #[serde(flatten)]
    pub data: SensorData,
    #[serde_as(as = "DurationMilliSeconds<u64>")]
    pub timestamp: Duration,
}

impl TimedSensorData {
    pub fn new(data: SensorData, start_time: SystemTime) -> Self {
        Self {
            data,
            timestamp: SystemTime::now()
                .duration_since(start_time)
                .expect("This is really bad"),
        }
    }
}

impl From<SensorData> for TimedSensorData {
    fn from(value: SensorData) -> Self {
        Self::new(value, SystemTime::now())
    }
}
