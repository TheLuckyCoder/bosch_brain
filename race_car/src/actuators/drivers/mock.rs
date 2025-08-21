use crate::actuators::drivers::{DutyCycle, PwmDriver};

pub struct MockPwm;

impl PwmDriver for MockPwm {
    fn set_duty_cycle(&mut self, _percentage: DutyCycle) {}

    fn turn_off(&mut self) {}
}
