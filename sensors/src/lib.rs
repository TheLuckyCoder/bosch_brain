#[cfg(target_os = "linux")]
use linux_embedded_hal::{Delay, I2cdev};
use mpu6050::Mpu6050;
use mpu6050::Mpu6050Error::{I2c, InvalidChipId};

pub struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

pub trait ImuSpecs {
    fn get_acceleration(&mut self) -> Vec3;

    fn get_gyroscope(&mut self) -> Vec3;
}

#[cfg(target_os = "linux")]
mod internal {
    use super::*;

    pub type Imu = Mpu6050<I2cdev>;

    impl ImuSpecs for Imu {
        fn get_acceleration(&mut self) -> Vec3 {
            let acc = self.get_acc().expect("Failed to get acc");
            Vec3::new(acc.x, acc.y, acc.z)
        }

        fn get_gyroscope(&mut self) -> Vec3 {
            let gyro = self.get_gyro().expect("Failed to get gyro");
            Vec3::new(gyro.x, gyro.y, gyro.z)
        }
    }
}

#[cfg(target_os = "windows")]
mod internal {
    use super::*;

    pub struct FakeImu {}

    impl ImuSpecs for FakeImu {
        fn get_acceleration(&mut self) -> Vec3 {
            Vec3::new(0.0, 0.0, 0.0)
        }

        fn get_gyroscope(&mut self) -> Vec3 {
            Vec3::new(0.0, 0.0, 0.0)
        }
    }
}

#[cfg(target_os = "linux")]
pub fn get_imu() -> Result<internal::Imu, String> {
    let i2c = I2cdev::new("/dev/i2c-1").map_err(|e| e.to_string())?;

    let mut imu = Mpu6050::new(i2c);
    let mut delay = Delay {};
    if let Err(e) = imu.init(&mut delay) {
        return Err(match e {
            I2c(e) => e.to_string(),
            InvalidChipId(id) => format!("Invalid chip id: {}", id),
        });
    }

    Ok(imu)
}

#[cfg(target_os = "windows")]
pub fn get_imu() -> Result<internal::FakeImu, String> {
    Ok(internal::FakeImu {})
}
