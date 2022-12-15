use bno055::Bno055;
use linux_embedded_hal::{Delay, I2cdev};

struct Imu(Bno055<I2cdev>);

impl Imu {
    pub fn new() -> Result<Self, String> {
        let i2c = I2cdev::new("/dev/i2c-1").map_err(|e| e.to_string())?;

        let mut imu = Bno055::new(i2c);
        let mut delay = Delay {};
        imu.init(&mut delay).map_err(|e| e.to_string())?;

        // Enable 9-degrees-of-freedom sensor fusion mode with fast magnetometer calibration
        imu.set_mode(bno055::BNO055OperationMode::NDOF, &mut delay)
            .map_err(|e| e.to_string())?;

        Ok(Self(imu))
    }
}
