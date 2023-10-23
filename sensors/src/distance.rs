use hc_sr04::{HcSr04, Unit};
use tracing::debug;

const TRIGGER: u8 = 24;
const ECHO: u8 = 23;

pub struct DistanceSensor(HcSr04);

impl DistanceSensor {
    pub fn new(temp: f32) -> Result<Self, String> {
        HcSr04::new(TRIGGER, ECHO, Some(temp))
            .map(Self)
            .map_err(|e| e.to_string())
    }

    pub fn start_calibration(&self) {
        debug!("Not yet implemented");
    }

    /**
     * Returns the distance in centimeters.
     */
    pub fn get_distance_cm(&mut self) -> Option<f32> {
        match self.0.measure_distance(Unit::Centimeters) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("Failed to read distance sensor: {e}");
                None
            }
        }
    }
}
