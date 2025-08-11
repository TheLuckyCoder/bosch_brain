use std::convert::TryInto;
use std::path::PathBuf;

use anyhow::Context;
use bno055::{BNO055Calibration, BNO055OperationMode, Bno055, BNO055_CALIB_SIZE};
use linux_embedded_hal::{Delay, I2cdev};
use mint::{Quaternion, Vector3};
use tracing::{error, info, warn};

use shared::math::AlmostEquals;

use crate::name::SensorName;
use crate::{HardwareSensor, SensorData};

/// Wrapper for the BNO055 sensor
pub struct ImuSensor {
    imu: Bno055<I2cdev>,
    calibration_file_path: PathBuf,
}

impl ImuSensor {
    const BNO_FILE: &'static str = "bno.bin";

    pub fn new(mut calibration_folder: PathBuf, use_alt_address: bool) -> anyhow::Result<Self> {
        calibration_folder.push(Self::BNO_FILE);

        let i2c = I2cdev::new("/dev/i2c-1").context("Failed to open I2C device")?;

        let mut imu = if use_alt_address {
            Bno055::new(i2c).with_alternative_address()
        } else {
            Bno055::new(i2c)
        };

        let mut delay = Delay {};
        imu.init(&mut delay).context("Failed to init IMU")?;

        if let Ok(file_buffer) = std::fs::read(calibration_folder.as_path()) {
            let buffer: [u8; BNO055_CALIB_SIZE] = vec_to_array(file_buffer);
            let calib = BNO055Calibration::from_buf(&buffer);
            imu.set_calibration_profile(calib, &mut delay)
                .context("Failed to load calibration profile")?;
            info!("IMU Calibration was loaded");
        }

        imu.set_mode(BNO055OperationMode::NDOF, &mut delay)
            .context("Failed to set IMU mode")?;

        Ok(Self {
            imu,
            calibration_file_path: calibration_folder,
        })
    }


    pub fn get_acceleration(&mut self) -> Vector3<f32> {
        self.imu.linear_acceleration().unwrap_or_else(|e| {
            error!("IMU probably not in fusion mode: {e}");
            Vector3::from([f32::NAN; 3])
        })
    }

    pub fn get_quaternion(&mut self) -> Quaternion<f32> {
        self.imu
            .quaternion()
            .map(|q| {
                let vec = &q.v;
                if (vec.x.powi(2) + vec.y.powi(2) + vec.z.powi(2) + q.s.powi(2))
                    .almost_equals(1.0, 0.1)
                {
                    q
                } else {
                    warn!("IMU reported wrong value");
                    Quaternion::from([f32::NAN; 4])
                }
            })
            .unwrap_or_else(|e| {
                error!("IMU probably not in fusion mode: {e}");
                Quaternion::from([f32::NAN; 4])
            })
    }
}

impl HardwareSensor for ImuSensor {
    fn name(&self) -> SensorName {
        SensorName::Imu
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Imu {
            quaternion: self.get_quaternion().into(),
            acceleration: self.get_acceleration().into(),
        }
    }

    fn read_debug(&mut self) -> String {
        let status = self
            .imu
            .get_calibration_status()
            .expect("Failed to get calibration status");

        format!(
            "IMU Calibration Status sys: {} gyr: {} acc: {} mag: {}",
            status.sys, status.gyr, status.acc, status.mag
        )
    }

    fn end_calibration(&mut self) -> anyhow::Result<()> {
        let mut delay = Delay {};

        let calibration = self
            .imu
            .calibration_profile(&mut delay)
            .context("Failed to get calibration result")?;

        std::fs::write(self.calibration_file_path.as_path(), calibration.as_bytes())
            .context("Failed to save calibration")?;

        info!("IMU calibration is saved");
        Ok(())
    }
}

fn vec_to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}
