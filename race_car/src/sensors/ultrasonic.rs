use hc_sr04::{HcSr04, Unit};

use crate::sensors::{BasicSensor, SensorData, SensorName};

const TRIGGER: u8 = 24;
const ECHO: u8 = 23;

/// Wrapper for the HC-SR04 sensor
pub struct UltrasonicSensor(HcSr04);

impl UltrasonicSensor {
    pub const NAME: &'static str = "Ultrasonic";

    pub fn new(temp: f32) -> Result<Self, String> {
        HcSr04::new(TRIGGER, ECHO, Some(temp))
            .map(Self)
            .map_err(|e| e.to_string())
    }

    /// Returns the distance in centimeters.
    pub fn get_distance_cm(&mut self) -> Option<f32> {
        self.0
            .measure_distance(Unit::Centimeters)
            .unwrap_or_else(|e| {
                tracing::error!("Failed to read distance sensor: {e}");
                None
            })
    }
}

impl BasicSensor for UltrasonicSensor {
    fn name(&self) -> SensorName {
        SensorName::Ultrasonic
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Distance(self.get_distance_cm().unwrap_or(f32::INFINITY))
    }
}
