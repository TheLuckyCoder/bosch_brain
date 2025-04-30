use anyhow::{Context, Result};
use rppal::gpio::{Gpio, OutputPin};
use rppal::pwm::{Channel, Polarity, Pwm};
use tracing::{error, info};

use crate::actuators::pwm::software_pwm::PiGpioDmaPwm;
use crate::actuators::pwm::{Percentage, PwmDriver};

pub struct PiZeroServoPwm {
    pwm: Pwm,
}

impl PiZeroServoPwm {
    pub fn new(channel: Channel, freq_hz: f64) -> Result<Self> {
        let mut pwm =
            Pwm::with_frequency(channel, freq_hz, 0.0, Polarity::Normal, true).context(format!(
                "creating PWM on channel {:?} with frequency {}",
                channel, freq_hz
            ))?;
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
    in1_pwm: PiGpioDmaPwm,
    in2_pwm: PiGpioDmaPwm,
}

impl PiZeroMotorPwm {
    pub fn new(in1_pin: u8, in2_pin: u8, freq_hz: u32) -> Result<Self> {
        let mut in1_pwm =
            PiGpioDmaPwm::new(in1_pin, 8888, freq_hz).context("creating motor PWM")?;
        let mut in2_pwm =
            PiGpioDmaPwm::new(in2_pin, 8888, freq_hz).context("creating motor PWM")?;
        in1_pwm
            .set_duty_cycle(Percentage::default())
            .context("setting IN1 duty cycle to 0")?;
        in2_pwm
            .set_duty_cycle(Percentage::default())
            .context("setting IN2 duty cycle to 0")?;
        info!("Set DutyCycle 0");

        Ok(Self { in1_pwm, in2_pwm })
    }
}

impl PwmDriver for PiZeroMotorPwm {
    fn set_duty_cycle(&mut self, percentage: Percentage) {
        // Set direction: HIGH = forward, LOW = backward
        let (in1, in2) = if percentage.is_positive {
            (percentage, Percentage::zero())
        } else {
            (Percentage::zero(), percentage)
        };

        if let Err(e) = self.in1_pwm.set_duty_cycle(in1) {
            error!("Failed to set IN1 duty cycle: {}", e);
            return;
        }
        if let Err(e) = self.in2_pwm.set_duty_cycle(in2) {
            error!("Failed to set IN2 duty cycle: {}", e);
            return;
        }

        if percentage.is_positive {
            info!("Direction: forward");
        } else {
            info!("Direction: backward");
        }
    }

    fn turn_off(&mut self) {
        /// IN1 = HIGH, IN2 = HIGH → brake (this is what we want)
        /// IN1 = LOW, IN2 = LOW → coast
        if let Err(e) = self.in1_pwm.set_duty_cycle(Percentage::zero()) {
            error!("Failed to turn off IN1 duty cycle: {}", e);
            return;
        }
        if let Err(e) = self.in2_pwm.set_duty_cycle(Percentage::zero()) {
            error!("Failed to turn off IN2 duty cycle: {}", e);
            return;
        }
    }
}

// pub struct PiZeroMotorPwm {
//     in1_pwm: Pwm,
//     in2_pwm: Pwm,
// }
// impl PiZeroMotorPwm {
//     pub fn new(pwm_channel_1: Channel, pwm_channel_2: Channel, freq_hz: f64) -> Result<Self> {
//         let in1_pwm = Pwm::with_frequency(pwm_channel_1, freq_hz, 0.0, Polarity::Normal, true)
//             .context("creating motor PWM")?;
//         in1_pwm.enable().context("enabling PWM")?;
//
//         let in2_pwm = Pwm::with_frequency(
//             pwm_channel_2,
//             freq_hz,
//             0.0,
//             Polarity::Normal,
//             true,
//         ).context("creating motor PWM")?;
//
//         in2_pwm.enable().context("enabling PWM")?;
//
//         // let gpio = Gpio::new().context("initializing GPIO")?;
//         // let dir_pin = gpio.get(dir_gpio).context("getting direction GPIO")?.into_output();
//
//         Ok(Self { in1_pwm, in2_pwm })
//     }
// }
//
// impl PwmDriver for PiZeroMotorPwm {
//     fn set_duty_cycle(&mut self, percentage: Percentage) {
//         let duty = percentage.value / 100.0;
//
//         // Set direction: HIGH = forward, LOW = backward
//         if percentage.is_positive {
//             self.in1_pwm.set_duty_cycle(duty).context("setting IN1 duty cycle");
//             self.in2_pwm.set_duty_cycle(0.0).context("setting IN2 duty cycle");
//             info!("Direction: forward");
//         } else {
//             self.in1_pwm.set_duty_cycle(0.0).context("setting IN1 duty cycle");
//             self.in2_pwm.set_duty_cycle(duty).context("setting IN2 duty cycle");
//             info!("Direction: backward");
//         }
//     }
//
//     fn turn_off(&mut self) {
//         /// IN1 = HIGH, IN2 = HIGH → brake (this is what we want)
//         /// IN1 = LOW, IN2 = LOW → coast
//         self.in1_pwm.set_duty_cycle(0.0).context("setting IN1 duty cycle");
//         self.in2_pwm.set_duty_cycle(0.0).context("setting IN2 duty cycle");
//     }
// }

// pub struct PiZeroMotorPwm {
//     in1: u32,
//     in2: u32,
// }
//
// impl PiZeroMotorPwm {
//     pub fn new(in1: u32, in2: u32, freq: u32) -> Result<Self> {
//         rust_pigpio::pwm::set_pwm_frequency(in1, freq).map_err(|e| anyhow::anyhow!("Error setting PWM frequency for IN1: {}", e))?;
//         rust_pigpio::pwm::set_pwm_frequency(in2, freq).map_err(|e| anyhow::anyhow!("Error setting PWM frequency for IN2: {}", e))?;
//         rust_pigpio::pwm::set_pwm_range(in1, 255).map_err(|e| anyhow::anyhow!("Error setting PWM range for IN1: {}", e))?;
//         rust_pigpio::pwm::set_pwm_range(in2, 255).map_err(|e| anyhow::anyhow!("Error setting PWM range for IN2: {}", e))?;
//         Ok(Self { in1, in2 })
//     }
// }
//
// impl PwmDriver for PiZeroMotorPwm {
//     fn set_duty_cycle(&mut self, percentage: Percentage) {
//         let duty = (percentage.value.abs() * 2.55).min(255.0) as u32;
//
//         if percentage.is_positive {
//             info!("Direction: forward");
//             rust_pigpio::pwm::pwm(self.in1, duty).unwrap_or_else(|e| error!("IN1 set failed: {}", e));
//             rust_pigpio::pwm::pwm(self.in2, 0).unwrap_or_else(|e| error!("IN2 set failed: {}", e));
//         } else {
//             info!("Direction: backward");
//             rust_pigpio::pwm::pwm(self.in1, 0).unwrap_or_else(|e| error!("IN1 set failed: {}", e));
//             rust_pigpio::pwm::pwm(self.in2, duty).unwrap_or_else(|e| error!("IN2 set failed: {}", e));
//         }
//
//         info!("Motor duty {:.1}%", percentage.value);
//     }
//
//     fn turn_off(&mut self) {
//         rust_pigpio::pwm::pwm(self.in1, 0).unwrap_or_else(|e| error!("IN1 off failed: {}", e));
//         rust_pigpio::pwm::pwm(self.in2, 0).unwrap_or_else(|e| error!("IN2 off failed: {}", e));
//         info!("Motor off");
//     }
// }

// use pigpio::{Pi, Mode};
//
// pub struct PiZeroMotorPwm {
//     pi: Pi,
//     in1: u32,
//     in2: u32,
// }
//
// impl PiZeroMotorPwm {
//     pub fn new(in1: u32, in2: u32) -> Result<Self> {
//         let pi = Pi::new().context("connecting to pigpiod")?;
//
//         pi.set_mode(in1, Mode::Output)?;
//         pi.set_mode(in2, Mode::Output)?;
//
//         pi.set_PWM_frequency(in1, 1000)?;
//         pi.set_PWM_frequency(in2, 1000)?;
//
//         Ok(Self { pi, in1, in2 })
//     }
// }
//
// impl PwmDriver for PiZeroMotorPwm {
//     fn set_duty_cycle(&mut self, percentage: Percentage) {
//         let duty = (percentage.value.abs() * 2.55).min(255.0) as u32;
//
//         if percentage.is_positive {
//             info!("Direction: forward");
//             self.pi.set_PWM_dutycycle(self.in1, duty).unwrap_or_else(|e| error!("IN1 set failed: {}", e));
//             self.pi.set_PWM_dutycycle(self.in2, 0).unwrap_or_else(|e| error!("IN2 set failed: {}", e));
//         } else {
//             info!("Direction: backward");
//             self.pi.set_PWM_dutycycle(self.in1, 0).unwrap_or_else(|e| error!("IN1 set failed: {}", e));
//             self.pi.set_PWM_dutycycle(self.in2, duty).unwrap_or_else(|e| error!("IN2 set failed: {}", e));
//         }
//
//         info!("Motor duty {:.1}%", percentage.value);
//     }
//
//     fn turn_off(&mut self) {
//         self.pi.set_PWM_dutycycle(self.in1, 0).unwrap_or_else(|e| error!("IN1 off failed: {}", e));
//         self.pi.set_PWM_dutycycle(self.in2, 0).unwrap_or_else(|e| error!("IN2 off failed: {}", e));
//         info!("Motor off");
//     }
// }
