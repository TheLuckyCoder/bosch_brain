use anyhow::Context;
use linux_embedded_hal::gpio_cdev::{Chip, LineRequestFlags};
use tracing::error;

use crate::sensors::drivers::ambience::AmbienceSensor;
use crate::sensors::drivers::gps::GpsSensor;
use crate::sensors::drivers::imu::ImuSensor;
use crate::sensors::drivers::ultrasonic::UltrasonicSensor;
use crate::sensors::drivers::velocity::VelocitySensor;
use crate::sensors::manager::SensorManager;

mod ambience;
mod gps;
mod imu;
mod ultrasonic;
mod velocity;

pub fn add_all_sensors(sensor_manager: &mut SensorManager) {
    ImuSensor::new()
        .map(|sensor| sensor_manager.add_sensor(sensor))
        .map_err(|e| error!("IMU failed to initialize: {e:?}"))
        .ok();
    sensor_manager.add_sensor(VelocitySensor::new(
        sensor_manager.get_data_receiver().add_stream(),
    ));
    UltrasonicSensor::new(21f32)
        .map(|sensor| sensor_manager.add_sensor(sensor))
        .map_err(|e| error!("Ultrasonic Sensor failed to initialize: {e:?}"))
        .ok();
    GpsSensor::new()
        .map(|sensor| sensor_manager.add_sensor(sensor))
        .map_err(|e| error!("GPS failed to initialize: {e}"))
        .ok();
    AmbienceSensor::new()
        .map(|sensor| sensor_manager.add_sensor(sensor))
        .map_err(|e| error!("AmbienceSensor failed to initialize: {e:?}"))
        .ok();
}

/// Helper function to set the board LED status
pub fn set_board_led_status(on: bool) -> anyhow::Result<()> {
    let mut chip = Chip::new("/dev/gpiochip0").context("Failed to open GPIO file")?;
    let output = chip.get_line(25).context("Failed to get GPIO PIN 25")?;

    output
        .request(LineRequestFlags::OUTPUT, on as u8, "blinky")
        .map(|_| ())
        .context("Failed to set GPIO PIN 25")
}
