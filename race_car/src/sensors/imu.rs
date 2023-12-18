use std::convert::TryInto;
use std::thread::sleep;
use std::time::Duration;

use crate::files::get_car_file;
use anyhow::Context;
use bno055::{BNO055Calibration, BNO055OperationMode, Bno055, BNO055_CALIB_SIZE};
use linux_embedded_hal::{Delay, I2cdev};
use tracing::{debug, error, info};

use crate::sensors::{BasicSensor, ImuData, SensorData};

pub struct Imu(Bno055<I2cdev>);

impl Imu {
    pub const NAME: &'static str = "IMU";
    const BNO_FILE: &'static str = "bno.bin";

    pub fn new() -> anyhow::Result<Self> {
        let i2c = I2cdev::new("/dev/i2c-1").context("Failed to open I2C device")?;

        let mut imu = Bno055::new(i2c).with_alternative_address();
        let mut delay = Delay {};

        imu.init(&mut delay).context("Failed to init IMU")?;
        imu.set_mode(BNO055OperationMode::NDOF, &mut delay)
            .context("Failed to set IMU mode")?;

        // Read saved calibration profile from MCUs NVRAM
        if let Ok(file_buffer) = std::fs::read(get_car_file(Self::BNO_FILE)) {
            let buffer: [u8; BNO055_CALIB_SIZE] = vec_to_array(file_buffer);

            // Apply calibration profile
            let calib = BNO055Calibration::from_buf(&buffer);
            imu.set_calibration_profile(calib, &mut delay)
                .context("Failed to load calibration profile")?;
            info!("IMU Calibration was loaded");
        }

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
        let mut delay = Delay {};
        self.0
            .set_mode(BNO055OperationMode::NDOF, &mut delay)
            .context("Failed to set IMU Mode")?;

        let file_path = get_car_file(Self::BNO_FILE);
        info!("BNO FIle: {}", file_path.display());

        // Start calibration and wait until it's complete.
        loop {
            let calibration_status = self
                .0
                .get_calibration_status()
                .context("Failed to get calibration status")?;
            info!("IMU Calibration Status: {:?}", calibration_status);

            // Check if all three calibration values are 3 to indicate full calibration.
            if calibration_status.sys == 3
                && calibration_status.gyr == 3
                && calibration_status.acc == 3
                && calibration_status.mag == 3
            {
                let calibration = self
                    .0
                    .calibration_profile(&mut delay)
                    .context("Failed to get calibration result")?;

                std::fs::write(file_path, calibration.as_bytes())
                    .context("Failed to save calibration")?;

                info!("Sensor is fully calibrated.");
                break; // Exit the loop once fully calibrated.
            }

            sleep(Duration::from_secs(1)); // Wait for a second before checking again.
        }

        Ok(())
    }

    pub fn get_acceleration(&mut self) -> mint::Vector3<f32> {
        self.0.linear_acceleration().unwrap_or_else(|e| {
            error!("IMU probably not in fusion mode: {e}");
            mint::Vector3::from([f32::NAN; 3])
        })
    }

    pub fn get_quaternion(&mut self) -> mint::Quaternion<f32> {
        self.0.quaternion().unwrap_or_else(|e| {
            error!("IMU probably not in fusion mode: {e}");
            mint::Quaternion::from([f32::NAN; 4])
        })
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

fn vec_to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}
