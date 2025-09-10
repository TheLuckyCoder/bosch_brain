use crate::actuators::configs::actuator_input_types::DutyCycle;
use rppal::pwm::{Pwm, Channel, Polarity};

pub mod mock;

pub mod pca9685_pwm;

pub mod pigpio_dma_pwm;
pub mod rppal_pwm;

pub trait PwmDriver: Send + 'static {
    fn set_duty_cycle(&mut self, duty_cycle: DutyCycle);

    fn turn_off(&mut self);

    fn force_low(&mut self) {
        self.turn_off();
    }
}
