use linux_embedded_hal::I2cdev;
use pwm_pca9685::{Address, Pca9685};
use serde::{Deserialize, Serialize};

pub use pwm_pca9685::Channel;

#[repr(usize)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Motor {
    Acceleration,
    Steering,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotorParams {
    pub percentage_minimum: f64,
    pub percentage_middle: f64,
    pub percentage_maximum: f64,
}

const DEFAULT_DC_MOTOR: MotorParams = MotorParams {
    percentage_minimum: 5.0,
    percentage_middle: 8.0,
    percentage_maximum: 11.0,
};

const DEFAULT_STEPPER_MOTOR: MotorParams = MotorParams {
    percentage_minimum: 5.0,
    percentage_middle: 8.0,
    percentage_maximum: 13.0,
};

fn map_from_percentage_to_12_bit_int(input: f64) -> u16 {
    tracing::info!("Final value %: {input}");
    // Maps an input floating-point number that is between 0.0-100.0 (percentage) to 0-4096

    // Ensure input is within the 0.0-100.0 range
    let clamped_input = input.clamp(0.0, 100.0);

    // Map clamped_input to the 0-4096 range
    (clamped_input * 40.96) as u16
}

struct MotorContents {
    params: MotorParams,
    bonnet_channel: Channel,
    last_value: f64,
    paused: bool,
}

pub struct MotorDriver {
    device: Pca9685<I2cdev>,
    contents: [MotorContents; 2],
}

impl MotorDriver {
    pub fn new() -> Result<Self, String> {
        let i2c = I2cdev::new("/dev/i2c-1").map_err(|e| format!("{e:?}"))?;
        let address = Address::default();
        let mut pwm = Pca9685::new(i2c, address).map_err(|e| format!("{e:?}"))?;

        // This corresponds to a frequency of 60 Hz.
        pwm.set_prescale(100).map_err(|e| format!("{e:?}"))?;

        // It is necessary to enable the device.
        pwm.enable().map_err(|e| format!("{e:?}"))?;

        Ok(Self {
            device: pwm,
            contents: [
                MotorContents {
                    params: DEFAULT_DC_MOTOR,
                    bonnet_channel: Channel::C0,
                    last_value: f64::INFINITY,
                    paused: false,
                },
                MotorContents {
                    params: DEFAULT_STEPPER_MOTOR,
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

        // if (input - *last_value).abs() < 10e-4 {
        //     return;
        // }

        *last_value = input;

        let params = &contents.params;
        let bonnet_channel = contents.bonnet_channel;

        // Maps an input number that is between -1 and 1 (float) to a percentage than can't be smaller than percentage_minimum and bigger than percentage_maximum
        // If the input is smaller than -1 or bigger than 1 it gives equivalent to it (percentage_minimum/maximum)
        let motor_input_percentage = if input != 0.0 {
            let max_min = if input > 0.0 {
                params.percentage_maximum
            } else {
                params.percentage_minimum
            };

            params.percentage_middle + input.abs() * (max_min - params.percentage_middle)
        } else {
            params.percentage_middle
        };

        // println!("Params: {params:?}");
        self.device
            .set_channel_on_off(
                bonnet_channel,
                0,
                map_from_percentage_to_12_bit_int(motor_input_percentage),
            )
            .expect("Failed to set motor input");
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
        self.stop_motor(Motor::Acceleration);
    }
}
