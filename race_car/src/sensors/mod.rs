//! Module containing all sensor abstraction classes

use std::time::SystemTime;

use crate::sensors::manager::SensorManager;
use crate::sensors::r#virtual::velocity::VelocitySensor;
use crate::utils::files::get_car_dir;
use sensors::drivers::{AmbienceSensor, GpsSensor, ImuSensor, UltrasonicSensor};
use sensors::SensorData;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::TimestampMilliSeconds;
use tracing::error;

pub mod manager;
pub mod mock;
mod r#virtual;

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

pub fn add_all_sensors(sensor_manager: &mut SensorManager) {
    let calibration_folder = get_car_dir();
    ImuSensor::new(calibration_folder)
        .map(|sensor| sensor_manager.add_sensor(sensor))
        .map_err(|e| error!("IMU failed to initialize: {e:?}"))
        .ok();
    UltrasonicSensor::new(21f32)
        .map(|sensor| sensor_manager.add_sensor(sensor))
        .map_err(|e| error!("Ultrasonic Sensor failed to initialize: {e:?}"))
        .ok();
    GpsSensor::new()
        .map(|sensor| sensor_manager.add_sensor(sensor))
        .map_err(|e| error!("GPS failed to initialize: {e}"))
        .ok();
    AmbienceSensor::new()
        .map(|sensor| sensor_manager.add_sensor(sensor))
        .map_err(|e| error!("AmbienceSensor failed to initialize: {e:?}"))
        .ok();
    sensor_manager.add_sensor(VelocitySensor::new(
        sensor_manager.get_data_receiver().add_stream(),
    ));
}
