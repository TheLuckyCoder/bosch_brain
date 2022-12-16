use linux_embedded_hal::{Delay, I2cdev};
use mpu6050::Mpu6050;
use mpu6050::Mpu6050Error::{I2c, InvalidChipId};

pub struct Imu(pub Mpu6050<I2cdev>);

impl Imu {
    pub fn new() -> Result<Self, String> {
        let i2c = I2cdev::new("/dev/i2c-1").map_err(|e| e.to_string())?;

        let mut imu = Mpu6050::new(i2c);
        let mut delay = Delay {};
        if let Err(e) = imu.init(&mut delay) {
            return match e {
                I2c(e) => Err(e.to_string()),
                InvalidChipId(id) => Err(format!("Invalid chip id: {}", id)),
            };
        }

        Ok(Self(imu))
    }
}
