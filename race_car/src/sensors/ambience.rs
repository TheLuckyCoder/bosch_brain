use crate::sensors::{BasicSensor, SensorData};
use anyhow::Context;
use linux_embedded_hal::{Delay, I2cdev};
use serde::Serialize;
use tracing::error;

/// Data from the ambience sensor
#[derive(Debug, Clone, Copy, Serialize)]
pub struct AmbienceData {
    pub temperature: f32,
    pub humidity: f32,
}

/// Wrapper for the HTU21DF sensor
pub struct AmbienceSensor(htu21df_sensor::Sensor<I2cdev>);

impl AmbienceSensor {
    pub const NAME: &'static str = "Ambience";

    pub fn new() -> anyhow::Result<AmbienceSensor> {
        let i2c = I2cdev::new("/dev/i2c-1").context("Failed to open I2C device")?;
        let mut delay = Delay {};

        let sensor = htu21df_sensor::Sensor::new(i2c, Some(&mut delay))
            .context("Failed to initialized ambience sensor")?;

        Ok(AmbienceSensor(sensor))
    }

    pub fn read_temperature(&mut self) -> f32 {
        let mut delay = Delay {};

        match self.0.measure_temperature(&mut delay) {
            Ok(temp) => temp.value(),
            Err(e) => {
                error!("Failed to read temperature: {e}");
                f32::NAN
            }
        }
    }

    pub fn read_humidity(&mut self) -> f32 {
        let mut delay = Delay {};

        match self.0.measure_humidity(&mut delay) {
            Ok(humidity) => humidity.value(),
            Err(e) => {
                error!("Failed to read humidity: {e}");
                f32::NAN
            }
        }
    }
}

impl BasicSensor for AmbienceSensor {
    fn name(&self) -> &'static str {
        AmbienceSensor::NAME
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Ambience(AmbienceData {
            temperature: self.read_temperature(),
            humidity: self.read_humidity(),
        })
    }
}
