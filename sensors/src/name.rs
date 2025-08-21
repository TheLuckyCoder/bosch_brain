use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{AsRefStr, EnumIter, IntoEnumIterator, IntoStaticStr};
use tracing::info;

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
    Gps,
    Imu,
    Bno085,
    Ultrasonic,
    VirtualVelocity,
    OpticalRotaryEncoder,
    QuadratureEncoder,
}

impl FromStr for SensorName {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SensorName::iter()
            .find(|v| v.as_ref().eq_ignore_ascii_case(s))
            .ok_or_else(|| {
                info!("Unknown sensor name: {}", s);
                "No such Sensor exists"
            })
    }
}

impl Display for SensorName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.into())
    }
}
