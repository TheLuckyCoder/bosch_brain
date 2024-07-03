use crate::name::SensorName;
use crate::{HardwareSensor, SensorData};
use hc_sr04::{HcSr04, Unit};

/// Wrapper for the HC-SR04 sensor
pub struct UltrasonicSensor(HcSr04);

impl UltrasonicSensor {
    pub fn new(temp: f32) -> Result<Self, String> {
        const TRIGGER: u8 = 24;
        const ECHO: u8 = 23;

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

impl HardwareSensor for UltrasonicSensor {
    fn name(&self) -> SensorName {
        SensorName::Ultrasonic
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Ultrasonic(self.get_distance_cm().unwrap_or(f32::INFINITY))
    }
}
