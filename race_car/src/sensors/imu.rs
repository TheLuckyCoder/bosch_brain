use std::thread::sleep;
use std::time::Duration;

use anyhow::Context;
use bno055::{BNO055OperationMode, Bno055};
use linux_embedded_hal::{Delay, I2cdev};
use tracing::{debug, error};

use crate::sensors::{BasicSensor, ImuData, SensorData};

pub struct Imu(Bno055<I2cdev>);

impl Imu {
    pub const NAME: &'static str = "IMU";

    pub fn new() -> anyhow::Result<Self> {
        let i2c = I2cdev::new("/dev/i2c-1").context("Failed to open I2C device")?;

        let mut imu = Bno055::new(i2c).with_alternative_address();
        let mut delay = Delay {};

        imu.init(&mut delay).context("Failed to init IMU")?;
        imu.set_mode(BNO055OperationMode::NDOF, &mut delay)
            .context("Failed to set IMU mode")?;

        Ok(Self(imu))
    }

    pub fn check_is_calibrated(&mut self) -> anyhow::Result<bool> {
        let calibration_status = self
            .0
            .get_calibration_status()
            .context("Failed to get calibration status")?;

        debug!("IMU Calibration Status: {:?}", calibration_status);

        Ok(!(calibration_status.sys != 3
            || calibration_status.gyr != 3
            || calibration_status.acc != 3
            || calibration_status.mag != 3))
    }

    pub fn start_calibration(&mut self) -> anyhow::Result<()> {
        // Set the sensor to CONFIG mode for calibration.
        let mut delay = Delay {};
        self.0
            .set_mode(BNO055OperationMode::CONFIG_MODE, &mut delay)
            .context("Failed to set IMU Mode")?;

        // Start calibration and wait until it's complete.
        loop {
            let calibration_status = self
                .0
                .get_calibration_status()
                .context("Failed to get calibration status")?;
            debug!("IMU Calibration Status: {:?}", calibration_status);

            // Check if all three calibration values are 3 to indicate full calibration.
            if calibration_status.sys == 3
                && calibration_status.gyr == 3
                && calibration_status.acc == 3
            {
                debug!("Sensor is fully calibrated.");
                break; // Exit the loop once fully calibrated.
            }

            sleep(Duration::from_secs(1)); // Wait for a second before checking again.
        }

        self.0
            .set_mode(BNO055OperationMode::NDOF, &mut delay)
            .context("Failed to set IMU Mode")
    }

    pub fn get_acceleration(&mut self) -> mint::Vector3<f32> {
        match self.0.linear_acceleration() {
            Ok(v) => v,
            Err(e) => {
                error!("IMU probably not in fusion mode: {e}");
                mint::Vector3::from([f32::NAN; 3])
            }
        }
    }

    pub fn get_quaternion(&mut self) -> mint::Quaternion<f32> {
        match self.0.quaternion() {
            Ok(v) => v,
            Err(e) => {
                error!("IMU probably not in fusion mode: {e}");
                mint::Quaternion::from([f32::NAN; 4])
            }
        }
    }
}

impl BasicSensor for Imu {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Imu(ImuData {
            quaternion: self.get_quaternion(),
            acceleration: self.get_acceleration(),
        })
    }
}
