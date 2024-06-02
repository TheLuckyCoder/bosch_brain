use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{AsRefStr, EnumIter, IntoStaticStr};

pub use pwm::mock::*;

use crate::actuators::manager::ActuatorManager;
use crate::actuators::motor_drivers::{SteeringMotorParams, VelocityMotorParams};
use crate::actuators::pwm::pca9685::Pca9685Pwm;
use crate::actuators::pwm_motor_driver::PwmMotorDriver;

pub mod manager;
pub mod motor_drivers;
pub mod pwm;
pub mod pwm_motor_driver;

pub trait ActuatorDriver: Send + 'static {
    fn name(&self) -> ActuatorName;

    fn set_value(&mut self, value: f64);

    fn stop(&mut self) {
        self.set_value(0.0);
    }

    fn pause(&mut self) {}

    fn resume(&mut self) {}

    fn is_paused(&self) -> bool {
        false
    }

    fn get_config_json(&self) -> Option<String> {
        None
    }

    fn get_config_html(&self) -> Option<String> {
        None
    }

    fn save_config(&mut self, config: String) -> std::io::Result<()> {
        Err(std::io::Error::other(format!(
            "Not implemented, config ({config}) is simply ignored"
        )))
    }
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
    PartialOrd,
    Ord,
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

pub fn add_all_actuators(manager: &mut ActuatorManager) {
    let steering_motor= PwmMotorDriver::new(
        ActuatorName::SteeringMotor,
        Pca9685Pwm::new("/dev/i2c-1", pwm_pca9685::Channel::C1).unwrap(),
        SteeringMotorParams {
            min: 7.2,
            middle: 9.07,
            max: 10.95,
        },
    );
    manager.add_actuator(steering_motor);


    let mut speed_motor = PwmMotorDriver::new(
        ActuatorName::SpeedMotor,
        Pca9685Pwm::new("/dev/i2c-1", pwm_pca9685::Channel::C0).unwrap(),
        VelocityMotorParams {
            min: 8.2,
            lower_middle: 8.6,
            upper_middle: 9.5,
            max: 9.7,
        },
    );
    speed_motor.set_inverse_direction(true);
    manager.add_actuator(speed_motor);
}

pub fn add_all_mock_actuators(manager: &mut ActuatorManager) {
    manager.add_actuator(PwmMotorDriver::new(
        ActuatorName::SteeringMotor,
        MockPwm,
        SteeringMotorParams {
            min: 7.2,
            middle: 9.07,
            max: 10.95,
        },
    ));
    manager.add_actuator(PwmMotorDriver::new(
        ActuatorName::SpeedMotor,
        MockPwm,
        VelocityMotorParams {
            min: 8.2,
            lower_middle: 8.6,
            upper_middle: 9.5,
            max: 9.7,
        },
    ));
}
