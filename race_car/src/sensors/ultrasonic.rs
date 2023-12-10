use hc_sr04::{HcSr04, Unit};
use tracing::debug;

use crate::sensors::{BasicSensor, SensorData};

const TRIGGER: u8 = 24;
const ECHO: u8 = 23;

pub struct UltrasonicSensor(HcSr04);

impl UltrasonicSensor {
    pub fn new(temp: f32) -> Result<Self, String> {
        HcSr04::new(TRIGGER, ECHO, Some(temp))
            .map(Self)
            .map_err(|e| e.to_string())
    }

    pub fn start_calibration(&self) {
        debug!("Not yet implemented");
    }

    ///
    /// Returns the distance in centimeters.
    ///
    pub fn get_distance_cm(&mut self) -> Option<f32> {
        match self.0.measure_distance(Unit::Centimeters) {
            Ok(value) => value,
            Err(e) => {
                tracing::error!("Failed to read distance sensor: {e}");
                None
            }
        }
    }
}

impl BasicSensor for UltrasonicSensor {
    fn name(&self) -> &'static str {
        "Ultrasonic"
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Distance(self.get_distance_cm().unwrap_or(f32::INFINITY))
    }
}
