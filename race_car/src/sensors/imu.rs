use anyhow::{anyhow, Context};
use bno055::{BNO055OperationMode, Bno055};
use linux_embedded_hal::{Delay, I2cdev};
use std::thread::sleep;
use std::time::Duration;
use tracing::debug;

pub struct GenericImu(Bno055<I2cdev>);

impl GenericImu {
    /**
     * THIS SHOULD ONLY BE CALLED ONCE
     */
    pub fn new() -> anyhow::Result<Self> {
        let i2c =
            I2cdev::new("/dev/i2c-1").map_err(|e| anyhow!("Failed to open I2C device: {e}"))?;

        let mut imu = Bno055::new(i2c).with_alternative_address();
        let mut delay = Delay {};

        imu.init(&mut delay)
            .map_err(|e| anyhow!("Failed to init IMU: {e}"))?;
        imu.set_mode(BNO055OperationMode::NDOF, &mut delay)
            .map_err(|e| anyhow!("Failed to set IMU mode: {e}"))?;

        Ok(Self(imu))
    }

    pub fn get_temperature(&mut self) -> anyhow::Result<i8> {
        self.0.temperature().context("Failed to get temperature")
    }

    pub fn is_calibrated(&mut self) -> anyhow::Result<bool> {
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
            Err(e) => panic!("Sensor probably not in fusion mode: {e}"),
        }
    }

    pub fn get_quaternion(&mut self) -> mint::Quaternion<f32> {
        match self.0.quaternion() {
            Ok(v) => v,
            Err(e) => panic!("Sensor probably not in fusion mode: {e}"),
        }
    }
}
