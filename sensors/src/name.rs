use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{AsRefStr, EnumIter, IntoStaticStr};

#[derive(
    Debug,
    Clone,
    Copy,
    DeserializeFromStr,
    SerializeDisplay,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    EnumIter,
    IntoStaticStr,
    AsRefStr,
)]
pub enum SensorName {
    Ambience,
    Gps,
    Imu,
    Ultrasonic,
    Velocity,
}

impl FromStr for SensorName {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "imu" => Ok(SensorName::Imu),
            "ultrasonic" => Ok(SensorName::Ultrasonic),
            "gps" => Ok(SensorName::Gps),
            "velocity" => Ok(SensorName::Velocity),
            "ambience" => Ok(SensorName::Ambience),
            _ => Err("No such Sensor exists"),
        }
    }
}

impl Display for SensorName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.into())
    }
}
