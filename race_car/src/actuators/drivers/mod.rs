use crate::actuators::configs::actuator_input_types::DutyCycle;
use rppal::pwm::{Pwm, Channel, Polarity};

pub mod mock;

pub mod pca9685_pwm;

pub mod pigpio_dma_pwm;

pub trait PwmDriver: Send + 'static {
    fn set_duty_cycle(&mut self, duty_cycle: DutyCycle);

    fn turn_off(&mut self);

    fn force_low(&mut self) {
        self.turn_off();
    }
}


pub struct RppalPwmDriver {
    pwm: Pwm,
}

impl RppalPwmDriver {
    pub fn new(channel: Channel, frequency_hz: f64) -> rppal::pwm::Result<Self> {
        let pwm = Pwm::with_frequency(channel, frequency_hz, 0.0, Polarity::Normal, true)?;
        Ok(Self { pwm })
    }
}

impl PwmDriver for RppalPwmDriver {
    fn set_duty_cycle(&mut self, duty_cycle: DutyCycle) {
        // Re-enable PWM if previously disabled
        let duty_fraction = duty_cycle.as_fraction();
        self.pwm
            .set_duty_cycle(duty_fraction)
            .expect("Failed to set PWM duty cycle");
    }

    fn turn_off(&mut self) {
        self.pwm.disable().expect("Failed to disable PWM");
    }
}
