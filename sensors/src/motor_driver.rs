use linux_embedded_hal::I2cdev;
use pwm_pca9685::{Address, Channel, Pca9685};

struct MotorSettings {
    bonnet_channel: Channel,
    percentage_minimum: f64,
    percentage_middle: f64,
    percentage_maximum: f64,
}

const STEPPER_MOTOR: MotorSettings = MotorSettings {
    bonnet_channel: Channel::C1,
    percentage_minimum: 5.0,
    percentage_middle: 8.0,
    percentage_maximum: 13.0,
};
const DC_MOTOR: MotorSettings = MotorSettings {
    bonnet_channel: Channel::C0,
    percentage_minimum: 5.0,
    percentage_middle: 8.0,
    percentage_maximum: 11.0,
};

fn map_from_percentage_to_12_bit_int(input: f64) -> u16 {
    // Maps an input floating-point number that is between 0.0-100.0 (percentage) to 0-4096

    // Ensure input is within the 0.0-100.0 range
    let clamped_input = input.clamp(0.0, 100.0);

    // Map clamped_input to the 0-4096 range
    (clamped_input * 40.96) as u16
}

pub struct MotorDriver(Pca9685<I2cdev>);

impl MotorDriver {
    pub fn new() -> Result<Self, String> {
        let i2c = I2cdev::new("/dev/i2c-1").map_err(|e| format!("{e:?}"))?;
        let address = Address::default();
        let mut pwm = Pca9685::new(i2c, address).map_err(|e| format!("{e:?}"))?;

        // This corresponds to a frequency of 60 Hz.
        pwm.set_prescale(100).map_err(|e| format!("{e:?}"))?;

        // It is necessary to enable the device.
        pwm.enable().map_err(|e| format!("{e:?}"))?;

        Ok(Self(pwm))
    }

    /**
     * @param angle [-1.0, 1.0]
     */
    pub fn set_steering_angle(&mut self, angle: f64) {
        self.set_motor_input(angle, &STEPPER_MOTOR)
    }

    /**
     * @param angle [-1.0, 1.0]
     */
    pub fn set_acceleration(&mut self, acceleration: f64) {
        self.set_motor_input(acceleration, &DC_MOTOR)
    }

    fn set_motor_input(&mut self, input: f64, motor_settings: &MotorSettings) {
        //Maps an input number that is between -1 and 1 (float) to a percentage than can't be smaller than percentage_minimum and bigger than percentage_maximum
        //If the input is smaller than -1 or bigger than 1 it gives equivalent to it (percentage_minimum/maximum)
        let clamped_input = input.clamp(-1.0, 1.0);

        let motor_input_percentage: f64 = if clamped_input == 0.0 {
            motor_settings.percentage_middle
        } else if (-1.0..0.0).contains(&clamped_input) {
            clamped_input * (motor_settings.percentage_middle - motor_settings.percentage_minimum)
                + motor_settings.percentage_minimum
        } else if 0.0 < clamped_input && clamped_input <= 1.0 {
            clamped_input * (motor_settings.percentage_maximum - motor_settings.percentage_middle)
                + motor_settings.percentage_middle
        } else {
            motor_settings.percentage_middle
        };

        self.0
            .set_channel_on_off(
                motor_settings.bonnet_channel,
                0,
                map_from_percentage_to_12_bit_int(motor_input_percentage),
            )
            .expect("Failed to set motor input");
    }
}

impl Drop for MotorDriver {
    fn drop(&mut self) {
        self.set_acceleration(0.0);
    }
}
