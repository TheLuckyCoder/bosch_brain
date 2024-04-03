use serde::{Deserialize, Serialize};
use crate::actuator::{Actuator, Pwm};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorParams {
    pub min: f64,
    pub lower_middle: f64,
    pub upper_middle: f64,
    pub max: f64,
}

pub struct MotorDriver<P: Pwm> {
    pwm: P,
    params: MotorParams,
    last_value: f64,
    paused: bool,
    inverse_direction: bool,
}

impl<P: Pwm> MotorDriver<P> {
    pub fn new(pwm: P, params: MotorParams, inverse_direction: bool) -> Self {
        Self {
            pwm,
            params,
            last_value: f64::INFINITY,
            paused: false,
            inverse_direction
        }
    }
}

impl<P: Pwm> Actuator for MotorDriver<P> {
    fn set_value(&mut self, value: f64) {
        // Steering motor should turn right when given a positive value
        let input = value.clamp(-1.0, 1.0) * if self.inverse_direction { -1.0 } else { 1.0 };

        if self.paused && (input - self.last_value).abs() < 10e-6 {
            return;
        }

        self.last_value = input;

        let params = &self.params;

        // Maps an input number that is between -1 and 1 (float) to a percentage than can't be smaller than percentage_minimum and bigger than percentage_maximum
        // If the input is smaller than -1 or bigger than 1 it gives equivalent to it (percentage_minimum/maximum)
        let motor_input_percentage = if input != 0.0 {
            if input > 0.0 {
                params.upper_middle + input * (params.max - params.upper_middle)
            } else {
                params.lower_middle + -input * (params.min - params.lower_middle)
            }
        } else {
            (params.lower_middle + params.upper_middle) / 2.0
        };

        self.pwm.set_duty_cycle(motor_input_percentage);
    }

    fn stop(&mut self) {
        self.pwm.turn_off();
    }

    fn pause(&mut self) {
        self.paused = true;
        self.stop();
    }

    fn resume(&mut self) {
        self.paused = false;
    }
}

impl<P: Pwm> Drop for MotorDriver<P> {
    fn drop(&mut self) {
        self.stop()
    }
}
