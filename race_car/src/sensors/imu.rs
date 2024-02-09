use std::convert::TryInto;

use anyhow::Context;
use bno055::{BNO055Calibration, BNO055OperationMode, Bno055, BNO055_CALIB_SIZE};
use linux_embedded_hal::{Delay, I2cdev};
use mint::{Quaternion, Vector3};
use serde::Serialize;
use shared::math::AlmostEquals;
use tracing::{error, info, warn};

use crate::files::get_car_file;
use crate::sensors::{BasicSensor, SensorData};

/// Data from the IMU sensor
#[derive(Debug, Clone, Serialize)]
pub struct ImuData {
    pub quaternion: Quaternion<f32>,
    pub acceleration: Vector3<f32>,
}

/// Wrapper for the BNO055 sensor
pub struct Imu(Bno055<I2cdev>);

impl Imu {
    pub const NAME: &'static str = "IMU";

    const BNO_FILE: &'static str = "bno.bin";

    pub fn new() -> anyhow::Result<Self> {
        let i2c = I2cdev::new("/dev/i2c-1").context("Failed to open I2C device")?;

        let mut imu = Bno055::new(i2c).with_alternative_address();
        let mut delay = Delay {};

        imu.init(&mut delay).context("Failed to init IMU")?;

        if let Ok(file_buffer) = std::fs::read(get_car_file(Self::BNO_FILE)) {
            let buffer: [u8; BNO055_CALIB_SIZE] = vec_to_array(file_buffer);

            // Apply calibration profile
            let calib = BNO055Calibration::from_buf(&buffer);
            imu.set_calibration_profile(calib, &mut delay)
                .context("Failed to load calibration profile")?;
            info!("IMU Calibration was loaded");
        }

        imu.set_mode(BNO055OperationMode::NDOF, &mut delay)
            .context("Failed to set IMU mode")?;

        Ok(Self(imu))
    }

    pub fn get_acceleration(&mut self) -> Vector3<f32> {
        self.0.linear_acceleration().unwrap_or_else(|e| {
            error!("IMU probably not in fusion mode: {e}");
            Vector3::from([f32::NAN; 3])
        })
    }

    pub fn get_quaternion(&mut self) -> Quaternion<f32> {
        self.0
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

    fn read_debug(&mut self) -> String {
        let status = self
            .0
            .get_calibration_status()
            .expect("Failed to get calibration status");

        format!(
            "IMU Calibration Status sys: {} gyr: {} acc: {} mag: {}",
            status.sys, status.gyr, status.acc, status.mag
        )
    }

    fn save_config(&mut self) -> anyhow::Result<()> {
        let mut delay = Delay {};

        let calibration = self
            .0
            .calibration_profile(&mut delay)
            .context("Failed to get calibration result")?;

        let file_path = get_car_file(Self::BNO_FILE);
        std::fs::write(file_path, calibration.as_bytes()).context("Failed to save calibration")?;

        info!("IMU calibration is saved");
        Ok(())
    }
}

fn vec_to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}
