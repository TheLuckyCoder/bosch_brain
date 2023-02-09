use hc_sr04::{HcSr04, Unit};

pub trait DistanceSensor {
    fn get_distance(&mut self) -> Result<Option<f32>, String>;
}

impl DistanceSensor for HcSr04 {
    /**
     * Returns the distance in centimeters.
     */
    fn get_distance(&mut self) -> Result<Option<f32>, String> {
        self.measure_distance(Unit::Centimeters)
            .map_err(|e| format!("Failed to read distance sensor: {e}"))
    }
}

pub(crate) fn get_distance_sensor() -> Result<impl DistanceSensor, String> {
    Ok(HcSr04::new(18, 24, Some(20_f32)).map_err(|e| e.to_string())?)
}
