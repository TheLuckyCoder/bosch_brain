use crate::actuators::Pwm;

pub struct MockPwm;

impl Pwm for MockPwm {
    fn set_duty_cycle(&mut self, _percentage: f64) {}

    fn turn_off(&mut self) {}
}
