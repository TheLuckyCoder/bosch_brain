use rppal::pwm::{Channel, Polarity, Pwm};
use crate::actuators::configs::actuator_input_types::DutyCycle;
use crate::actuators::drivers::PwmDriver;

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
        let duty_fraction = duty_cycle.as_unsigned_fraction();
        self.pwm
            .set_duty_cycle(duty_fraction)
            .expect("Failed to set PWM duty cycle");
    }

    fn turn_off(&mut self) {
        self.pwm.disable().expect("Failed to disable PWM");
    }
}

pub struct RppalWithDirPwmDriver {
    pwm: Pwm,
    dir_pin: rppal::gpio::OutputPin,
}

impl RppalWithDirPwmDriver {
    pub fn new(
        pwm_channel: Channel,
        frequency_hz: f64,
        dir_pin: u8,
    ) -> rppal::pwm::Result<Self> {
        let pwm = Pwm::with_frequency(pwm_channel, frequency_hz, 0.0, Polarity::Normal, true)?;
        let gpio = rppal::gpio::Gpio::new().expect("Failed to access GPIO");
        let dir_pin = gpio.get(dir_pin).expect("Failed to get dir pin").into_output();
        Ok(Self { pwm, dir_pin })
    }
}

impl PwmDriver for RppalWithDirPwmDriver {
    fn set_duty_cycle(&mut self, duty_cycle: DutyCycle) {
        if duty_cycle.sign < 0 {
            self.dir_pin.set_low();
        } else {
            self.dir_pin.set_high();
        }
        print!("Setting duty cycle to: {:?} and dir pin to: {:?}\n", duty_cycle, self.dir_pin.is_set_high());
        let duty_fraction = duty_cycle.as_unsigned_fraction();
        print!("Duty fraction: {}\n", duty_fraction);
        self.pwm
            .set_duty_cycle(duty_fraction)
            .expect("Failed to set PWM duty cycle");
    }

    fn turn_off(&mut self) {
        self.pwm.disable().expect("Failed to disable PWM");
    }
}