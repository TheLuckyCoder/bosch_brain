use crate::sensors::{AmbienceData, BasicSensor, SensorData};
use anyhow::Context;
use linux_embedded_hal::{Delay, I2cdev};
use tracing::error;

pub struct Ambience(htu21df_sensor::Sensor<I2cdev>);

impl Ambience {
    pub const NAME: &'static str = "Ambience";

    pub fn new() -> anyhow::Result<Ambience> {
        let i2c = I2cdev::new("/dev/i2c-1").context("Failed to open I2C device")?;
        let mut delay = Delay {};

        let sensor = htu21df_sensor::Sensor::new(i2c, Some(&mut delay))
            .context("Failed to initialized ambience sensor")?;

        Ok(Ambience(sensor))
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

impl BasicSensor for Ambience {
    fn name(&self) -> &'static str {
        Ambience::NAME
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Ambience(AmbienceData {
            temperature: self.read_temperature(),
            humidity: self.read_humidity(),
        })
    }
}
