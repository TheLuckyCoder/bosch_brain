use std::fmt::{Display, Formatter};
use std::str::FromStr;
use rppal::pwm::Channel;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{AsRefStr, EnumIter, IntoStaticStr};

use crate::actuators::manager::ActuatorManager;
use crate::actuators::motor_driver_params::{SteeringMotorParams, VelocityMotorParams};
use crate::actuators::pwm::pca9685::Pca9685Pwm;
use crate::actuators::pwm_motor_driver::PwmMotorDriver;
use crate::actuators::pwm::lego_servo::{PiZeroMotorPwm, PiZeroServoPwm};
use crate::actuators::pizero_params::{DcMotorParams, ServoParams};

pub mod manager;
pub mod motor_driver_params;
pub mod pwm;
pub mod pwm_motor_driver;
pub mod pizero_params;

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

    let servo_params = ServoParams {
        physical_min_angle: -180.0,
        physical_max_angle:  180.0,
        angle_offset:          0.0,
        control_min_angle:    -90.0,
        control_max_angle:     90.0,
        servo_min_pulse:     500.0,
        servo_max_pulse:    2500.0,
        servo_freq_hz:        50.0,
    };
    manager.add_actuator(PwmMotorDriver::new(
        ActuatorName::SteeringMotor,
        PiZeroServoPwm::new(
            Channel::Pwm0,
            50.0,
        ).unwrap(),
        servo_params,
    ));

    let motor_params = DcMotorParams {
        supply_voltage: 12.0,
        target_max_voltage: 7.5,
        pwm_resolution: 10,
        pwm_freq_hz: 5001.0,
    };
    manager.add_actuator(PwmMotorDriver::new(
        ActuatorName::SpeedMotor,
        PiZeroMotorPwm::new(
            23,
            22,
            motor_params.pwm_freq_hz as u32,
        ).unwrap(),
        motor_params,
    ))
}

pub fn add_pi_zero_actuators(manager: &mut ActuatorManager) -> anyhow::Result<()> {
    // steering‐servo driver just needs a generic PWM at 50 Hz
    let servo_pwm = PiZeroServoPwm::new(Channel::Pwm0, 50.0)?;
    let servo_params = ServoParams {
        physical_min_angle: -180.0,
        physical_max_angle:  180.0,
        angle_offset:          0.0,
        control_min_angle:    -90.0,
        control_max_angle:     90.0,
        servo_min_pulse:     500.0,
        servo_max_pulse:    2500.0,
        servo_freq_hz:        50.0,
    };
    manager.add_actuator(PwmMotorDriver::new(
        ActuatorName::SteeringMotor,
        servo_pwm,
        servo_params,
    ));

    // …and similarly for your DRV8871 motor (you can factor accel, maxDuty into a MotorParams)
    Ok(())
}
