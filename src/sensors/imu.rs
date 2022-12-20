use linux_embedded_hal::{Delay, I2cdev};
use mpu6050::Mpu6050;
use mpu6050::Mpu6050Error::{I2c, InvalidChipId};

pub type Imu = Mpu6050<I2cdev>;

pub fn get_imu() -> Result<Imu, String> {
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
