use std::path::Path;

use anyhow::anyhow;
use linux_embedded_hal::I2cdev;
use crate::actuator::Pwm;

pub struct Pca9685Pwm {
    driver: pwm_pca9685::Pca9685<I2cdev>,
    channel: pwm_pca9685::Channel,
}

impl Pca9685Pwm {
    pub fn new(path: impl AsRef<Path>, channel: pwm_pca9685::Channel) -> anyhow::Result<Self> {
        let i2c = I2cdev::new(path).map_err(|e| anyhow!("{e:?}"))?;
        let address = pwm_pca9685::Address::default();
        let mut pwm = pwm_pca9685::Pca9685::new(i2c, address).map_err(|e| anyhow!("{e:?}"))?;

        // This corresponds to a frequency of 60 Hz.
        pwm.set_prescale(100).map_err(|e| anyhow!("{e:?}"))?;

        // It is necessary to enable the device.
        pwm.enable().map_err(|e| anyhow!("{e:?}"))?;

        Ok(Self { driver: pwm, channel })
    }
}

impl Pwm for Pca9685Pwm {
    fn set_duty_cycle(&mut self, percentage: f64) {
        self.driver
            .set_channel_on_off(
                self.channel,
                0,
                map_from_percentage_to_12_bit_int(percentage),
            )
            .expect("Failed to set pwm duty cycle");
    }

    fn turn_off(&mut self) {
        self.driver
            .set_channel_full_off(self.channel)
            .expect("Failed to turn off pwm");
    }
}

#[inline]
fn map_from_percentage_to_12_bit_int(input: f64) -> u16 {
    // Ensure input is within the 0.0-100.0 range
    let clamped_input = input.clamp(0.0, 100.0);

    // Map clamped_input to the 0-4096 range
    (clamped_input * 40.96) as u16
}
