use anyhow::{Context, Result};
use rppal::gpio::{Gpio, OutputPin};
use tracing::{error, info};
use rppal::pwm::{Channel, Pwm, Polarity};

use crate::actuators::pwm::{Percentage, PwmDriver};

pub struct PiZeroServoPwm {
    pwm: Pwm,
}

impl PiZeroServoPwm {
    pub fn new(channel: Channel, freq_hz: f64) -> Result<Self> {
        let mut pwm = Pwm::with_frequency(channel, freq_hz, 0.0, Polarity::Normal, true).context(
            format!("creating PWM on channel {:?} with frequency {}", channel, freq_hz)
        )?;
        pwm.enable().context("enabling PWM")?;
        Ok(Self { pwm })
    }
}

impl PwmDriver for PiZeroServoPwm {
    fn set_duty_cycle(&mut self, percentage: Percentage) {
        let frac = (percentage.value / 100.0).clamp(0.0, 1.0);
        if let Err(e) = self.pwm.set_duty_cycle(frac) {
            error!("PWM set_duty_cycle: {}", e);
        } else {
            info!("PWM duty {:.1}%", percentage.value);
        }
    }

    fn turn_off(&mut self) {
        if let Err(e) = self.pwm.disable() {
            error!("PWM disable: {}", e);
        } else {
            info!("PWM disabled");
        }
    }
}

pub struct PiZeroMotorPwm {
    pwm: Pwm,
    dir_pin: OutputPin,
}

impl PiZeroMotorPwm {
    pub fn new(pwm_channel: Channel, dir_gpio: u8, freq_hz: f64) -> Result<Self> {
        let pwm = Pwm::with_frequency(pwm_channel, freq_hz, 0.0, Polarity::Normal, true)
            .context("creating motor PWM")?;
        pwm.enable().context("enabling PWM")?;

        let gpio = Gpio::new().context("initializing GPIO")?;
        let dir_pin = gpio.get(dir_gpio).context("getting direction GPIO")?.into_output();

        Ok(Self { pwm, dir_pin })
    }
}

impl PwmDriver for PiZeroMotorPwm {
    fn set_duty_cycle(&mut self, percentage: Percentage) {
        let duty = percentage.value / 100.0;

        // Set direction: HIGH = forward, LOW = backward
        self.dir_pin.write(if percentage.is_positive {
            info!("Direction: forward");
            rppal::gpio::Level::High
        } else {
            info!("Direction: backward");
            rppal::gpio::Level::Low
        });

        if let Err(e) = self.pwm.set_duty_cycle(duty) {
            error!("PWM set_duty_cycle failed: {}", e);
        } else {
            info!("Motor duty {:.1}", duty);
        }
    }

    fn turn_off(&mut self) {
        /// IN1 = HIGH, IN2 = HIGH → brake (this is what we want)
        /// IN1 = LOW, IN2 = LOW → coast
        self.dir_pin.write(rppal::gpio::Level::Low);
        if let Err(e) = self.pwm.set_duty_cycle(0.0) {
            error!("PWM turn_off failed: {}", e);
        } else {
            info!("Motor off");
        }
    }
}
