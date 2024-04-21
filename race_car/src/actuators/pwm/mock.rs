use crate::actuators::pwm::{Percentage, Pwm};

pub struct MockPwm;

impl Pwm for MockPwm {
    fn set_duty_cycle(&mut self, _percentage: Percentage) {}

    fn turn_off(&mut self) {}
}
