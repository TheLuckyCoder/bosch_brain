use crate::actuator::Pwm;

pub struct MockPwm;

impl Pwm for MockPwm {
    fn set_duty_cycle(&mut self, percentage: f64) {}

    fn turn_off(&mut self) {}
}
