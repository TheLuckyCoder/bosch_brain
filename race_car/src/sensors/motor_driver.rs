use anyhow::anyhow;
use linux_embedded_hal::I2cdev;
use pwm_pca9685::{Address, Pca9685};
use serde::{Deserialize, Serialize};

use pwm_pca9685::Channel;

/// All the motors that can be controlled
#[repr(usize)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Motor {
    Speed,
    Steering,
}

/// The parameters for a motor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorParams {
    pub min: f64,
    pub lower_middle: f64,
    pub upper_middle: f64,
    pub max: f64,
}

/// Default Params for [Motor::Speed]
const DEFAULT_VELOCITY_MOTOR: MotorParams = MotorParams {
    min: 8.2,
    lower_middle: 8.6,
    upper_middle: 9.05,
    max: 9.08,
};

/// Default Params for [Motor::Steering]
const DEFAULT_STEERING_MOTOR: MotorParams = MotorParams {
    min: 7.2,
    lower_middle: 9.07,
    upper_middle: 9.07,
    max: 10.95,
};

/// Maps an input floating-point number that is between 0.0-100.0 (percentage) to 0-4096
fn map_from_percentage_to_12_bit_int(input: f64) -> u16 {
    // tracing::info!("Final value %: {input}");
    // Ensure input is within the 0.0-100.0 range
    let clamped_input = input.clamp(0.0, 100.0);

    // Map clamped_input to the 0-4096 range
    (clamped_input * 40.96) as u16
}

/// All the data needed for a motor
struct MotorContents {
    params: MotorParams,
    bonnet_channel: Channel,
    last_value: f64,
    paused: bool,
}

/// Handles the motor control abstracting over the PCA9685 PWM driver
pub struct MotorDriver {
    device: Pca9685<I2cdev>,
    contents: [MotorContents; 2],
}

impl MotorDriver {
    pub fn new() -> anyhow::Result<Self> {
        let i2c = I2cdev::new("/dev/i2c-1").map_err(|e| anyhow!("{e:?}"))?;
        let address = Address::default();
        let mut pwm = Pca9685::new(i2c, address).map_err(|e| anyhow!("{e:?}"))?;

        // This corresponds to a frequency of 60 Hz.
        pwm.set_prescale(100).map_err(|e| anyhow!("{e:?}"))?;

        // It is necessary to enable the device.
        pwm.enable().map_err(|e| anyhow!("{e:?}"))?;

        Ok(Self {
            device: pwm,
            contents: [
                MotorContents {
                    params: DEFAULT_VELOCITY_MOTOR,
                    bonnet_channel: Channel::C0,
                    last_value: f64::INFINITY,
                    paused: false,
                },
                MotorContents {
                    params: DEFAULT_STEERING_MOTOR,
                    bonnet_channel: Channel::C1,
                    last_value: f64::INFINITY,
                    paused: false,
                },
            ],
        })
    }

    pub fn set_motor_value(&mut self, motor: Motor, input: f64) {
        let input = input.clamp(-1.0, 1.0);
        let contents = &mut self.contents[motor as usize];
        let last_value = &mut contents.last_value;

        if contents.paused {
            return;
        }

        if (input - *last_value).abs() < 10e-6 {
            return;
        }

        *last_value = input;

        let params = &contents.params;
        let bonnet_channel = contents.bonnet_channel;

        // Maps an input number that is between -1 and 1 (float) to a percentage than can't be smaller than percentage_minimum and bigger than percentage_maximum
        // If the input is smaller than -1 or bigger than 1 it gives equivalent to it (percentage_minimum/maximum)
        let motor_input_percentage = if input != 0.0 {
            if input > 0.0 {
                params.upper_middle + input * (params.max - params.upper_middle)
            } else {
                params.lower_middle + -input * (params.min - params.lower_middle)
            }
        } else {
            (params.lower_middle + params.upper_middle) / 2.0
        };

        self.device
            .set_channel_on_off(
                bonnet_channel,
                0,
                map_from_percentage_to_12_bit_int(motor_input_percentage),
            )
            .expect("Failed to set motor input");
    }

    pub fn get_last_motor_value(&mut self, motor: Motor) -> f64 {
        self.contents[motor as usize].last_value
    }

    pub fn stop_motor(&mut self, motor: Motor) {
        let contents = &mut self.contents[motor as usize];

        self.device
            .set_channel_full_off(contents.bonnet_channel)
            .expect("Failed to set motor input");

        contents.last_value = f64::INFINITY;
    }

    pub fn pause_motor(&mut self, motor: Motor) {
        self.stop_motor(motor);
        self.contents[motor as usize].paused = true;
    }

    pub fn resume_motor(&mut self, motor: Motor) {
        self.contents[motor as usize].paused = false;
    }

    pub fn get_params(&self, motor: Motor) -> MotorParams {
        let contents = &self.contents[motor as usize];

        contents.params.clone()
    }

    pub fn set_params(&mut self, motor: Motor, params: MotorParams) {
        let contents = &mut self.contents[motor as usize];

        contents.last_value = f64::INFINITY;
        contents.params = params
    }
}

impl Drop for MotorDriver {
    fn drop(&mut self) {
        self.stop_motor(Motor::Speed);
    }
}
