use crate::actuators::pwm::{Percentage, PwmDriver};

pub struct MockPwm;

impl PwmDriver for MockPwm {
    fn set_duty_cycle(&mut self, _percentage: Percentage) {}

    fn turn_off(&mut self) {}
}
