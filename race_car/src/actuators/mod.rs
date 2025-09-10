use rppal::pwm::Channel;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use strum::{AsRefStr, EnumIter, IntoStaticStr};

use crate::actuators::drivers::pca9685_pwm::Pca9685Pwm;
use crate::actuators::drivers::pigpio_dma_pwm::PiGpioDmaPwm;
use crate::actuators::dual_channel_pwm_actuator::DualChannelPwmActuator;
use crate::actuators::manager::ActuatorManager;
use crate::actuators::single_channel_pwm_actuator::SingleChannelPwmActuator;
use configs::lego_car_config::{GeekServoConfig, LegoDcMotorConfig};
use configs::rc_car_driver_config::{EscMotorConfig, SteeringMotorConfig};

pub mod manager;
pub mod drivers;
mod configs;

mod single_channel_pwm_actuator;

mod dual_channel_pwm_actuator;

pub trait Actuator: Send + 'static {
    fn name(&self) -> ActuatorName;

    fn set_command(&mut self, value: f64);

    /// Stops the actuator from moving until the next command is set.
    fn stop(&mut self) {
        self.set_command(0.0);
    }

    /// Sets the actuator to a paused state, where it will not respond to commands until resumed.
    fn pause(&mut self) {}

    /// Resumes the actuator from a paused state, allowing it to respond to commands again.
    fn resume(&mut self) {}

    fn is_paused(&self) -> bool {
        false
    }

    fn get_config(&self) -> Option<String> {
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

pub fn add_all_lego_actuators(manager: &mut ActuatorManager) {
    let servo_params = GeekServoConfig {
        physical_min_angle: -180.0,
        physical_max_angle:  180.0,
        angle_offset:          0.0,
        control_min_angle:    -90.0,
        control_max_angle:     90.0,
        servo_min_pulse:     500.0,
        servo_max_pulse:    2500.0,
        servo_freq_hz:        50.0,
    };
    //
    // manager.add_actuator(SingleChannelPwmActuator::new(
    //     ActuatorName::SteeringMotor,
    //     RppalPwmDriver::new(Channel::Pwm1, servo_params.servo_freq_hz).unwrap(),
    //     servo_params,
    // ));

    (*manager).add_actuator(SingleChannelPwmActuator::new(
        ActuatorName::SteeringMotor,
        Pca9685Pwm::new("/dev/i2c-1", pwm_pca9685::Channel::C0).unwrap(),
        servo_params,
    ));

    let motor_params = LegoDcMotorConfig {
        supply_voltage: 12.0,
        target_max_voltage: 8.5,
        pwm_resolution: 10,
        pwm_freq_hz: 4000.0,
    };
    // manager.add_actuator(SingleChannelPwmActuator::new(
    //     ActuatorName::SpeedMotor,
    //     // PiGpioDmaPwm::new(23, 8888, motor_params.pwm_freq_hz as u32).unwrap(),
    //     // PiGpioDmaPwm::new(17, 8888, motor_params.pwm_freq_hz as u32).unwrap(),
    //     // RppalPwmDriver::new(Channel::Pwm0, motor_params.pwm_freq_hz).unwrap(),
    //     RppalWithDirPwmDriver::new(Channel::Pwm0, motor_params.pwm_freq_hz,23).unwrap(),
    //     motor_params,
    // ));
    manager.add_actuator(DualChannelPwmActuator::new(
        ActuatorName::SpeedMotor,
        Pca9685Pwm::new("/dev/i2c-1", pwm_pca9685::Channel::C1).unwrap(),
        Pca9685Pwm::new("/dev/i2c-1", pwm_pca9685::Channel::C2).unwrap(),
        motor_params,
    ));
}

pub fn add_all_rc_car_actuators(manager: &mut ActuatorManager) {
    (*manager).add_actuator(SingleChannelPwmActuator::new(
        ActuatorName::SpeedMotor,
        Pca9685Pwm::new("/dev/i2c-1", pwm_pca9685::Channel::C0).unwrap(),
        EscMotorConfig {
            min: 8.2,
            lower_middle: 8.6,
            upper_middle: 9.5,
            max: 9.7,
        },
    ));

    (*manager).add_actuator(SingleChannelPwmActuator::new(
        ActuatorName::SteeringMotor,
        Pca9685Pwm::new("/dev/i2c-1", pwm_pca9685::Channel::C1).unwrap(),
        SteeringMotorConfig {
            min: 7.2,
            middle: 9.07,
            max: 10.95,
        },
    ));
}

pub fn add_all_mock_actuators(manager: &mut ActuatorManager) {
    // manager.add_actuator(PwmMotorDriver::new(
    //     ActuatorName::SteeringMotor,
    //     MockPwm,
    //     SteeringMotorParams {
    //         min: 7.2,
    //         middle: 9.07,
    //         max: 10.95,
    //     },
    // ));
    // manager.add_actuator(PwmMotorDriver::new(
    //     ActuatorName::SpeedMotor,
    //     MockPwm,
    //     VelocityMotorParams {
    //         min: 8.2,
    //         lower_middle: 8.6,
    //         upper_middle: 9.5,
    //         max: 9.7,
    //     },
    // ));
}