use bno055::{BNO055OperationMode, Bno055};
use linux_embedded_hal::{Delay, I2cdev};
use std::thread::sleep;
use std::time::Duration;

pub struct GenericImu(Bno055<I2cdev>);

impl GenericImu {
    /**
     * THIS SHOULD ONLY BE CALLED ONCE
     */
    pub fn new() -> Result<Self, String> {
        let i2c = I2cdev::new("/dev/i2c-1").map_err(|e| e.to_string())?;

        let mut imu = Bno055::new(i2c).with_alternative_address();
        let mut delay = Delay {};

        imu.init(&mut delay).map_err(|e| e.to_string())?;

        Ok(Self(imu))
    }

    pub fn get_temperature(&mut self) -> Result<i8, String> {
        self.0.temperature().map_err(|e| e.to_string())
    }

    pub fn is_calibrated(&mut self) -> Result<bool, String> {
        let calibration_status = self.0.get_calibration_status().map_err(|e| e.to_string())?;

        log::debug!("IMU Calibration Status: {:?}", calibration_status);

        Ok(!(calibration_status.sys != 3
            || calibration_status.gyr != 3
            || calibration_status.acc != 3
            || calibration_status.mag != 3))
    }

    pub fn start_calibration(&mut self) -> Result<(), String> {
        // Set the sensor to CONFIG mode for calibration.
        let mut delay = Delay {};
        self.0
            .set_mode(BNO055OperationMode::CONFIG_MODE, &mut delay)
            .map_err(|e| e.to_string())?;

        // Start calibration and wait until it's complete.
        loop {
            let calibration_status = self.0.get_calibration_status().map_err(|e| e.to_string())?;
            log::debug!("IMU Calibration Status: {:?}", calibration_status);

            // Check if all three calibration values are 3 to indicate full calibration.
            if calibration_status.sys == 3
                && calibration_status.gyr == 3
                && calibration_status.acc == 3
            {
                log::debug!("Sensor is fully calibrated.");
                break; // Exit the loop once fully calibrated.
            }

            sleep(Duration::from_secs(1)); // Wait for a second before checking again.
        }

        self.0
            .set_mode(BNO055OperationMode::NDOF, &mut delay)
            .map_err(|e| e.to_string())
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
