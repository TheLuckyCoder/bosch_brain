use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{AsRefStr, EnumIter, IntoStaticStr};

mod actuators_manager;
mod pca9685_pwm;
mod motor_driver;

pub trait Actuator {
    fn set_value(&mut self, value: f64);
    
    fn stop(&mut self);
    
    fn pause(&mut self) {}
    
    fn resume(&mut self) {}
}

#[derive(
Debug,
Clone,
Copy,
DeserializeFromStr,
SerializeDisplay,
PartialEq,
Eq,
Hash,
EnumIter,
IntoStaticStr,
AsRefStr,
)]
pub enum ActuatorName {
    SpeedMotor,
    SteeringMotor,
}

impl FromStr for ActuatorName {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "speed" => Ok(ActuatorName::SpeedMotor),
            "steering" => Ok(ActuatorName::SteeringMotor),
            _ => Err("No such Sensor exists"),
        }
    }
}

impl Display for ActuatorName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.into())
    }
}


pub trait Pwm {
    fn set_duty_cycle(&mut self, percentage: f64);

    fn turn_off(&mut self);
}
