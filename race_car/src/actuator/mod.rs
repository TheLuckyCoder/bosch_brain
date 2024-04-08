use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{AsRefStr, EnumIter, IntoStaticStr};

pub mod manager;
pub mod pca9685_pwm;
pub mod motor_driver;
mod mock_pwm;

pub use mock_pwm::*;

pub trait Actuator : Send + 'static {
    fn name(&self) -> ActuatorName;
    
    fn set_value(&mut self, value: f64);
    
    fn stop(&mut self);
    
    fn pause(&mut self) {}
    
    fn resume(&mut self) {}
    
    fn is_paused(&self) -> bool { false }
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
    CameraRotation,
}

impl FromStr for ActuatorName {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "speedmotor" => Ok(ActuatorName::SpeedMotor),
            "steeringmotor" => Ok(ActuatorName::SteeringMotor),
            "camerarotation" => Ok(ActuatorName::CameraRotation),
            _ => Err("No such Actuator exists"),
        }
    }
}

impl Display for ActuatorName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.into())
    }
}


pub trait Pwm : Send + 'static {
    fn set_duty_cycle(&mut self, percentage: f64);

    fn turn_off(&mut self);
}
