use anyhow::Context;
use linux_embedded_hal::gpio_cdev::{Chip, LineRequestFlags};

mod gps;
mod imu;
mod ultrasonic;
mod opt_rotary_encoder;
mod quadrature_encoder;
mod bno085;

pub use gps::*;
pub use imu::*;
pub use ultrasonic::*;
pub use opt_rotary_encoder::*;
pub use quadrature_encoder::*;
pub use bno085::*;

/// Helper function to set the board LED status
pub fn set_board_led_status(on: bool) -> anyhow::Result<()> {
    let mut chip = Chip::new("/dev/gpiochip0").context("Failed to open GPIO file")?;
    let output = chip.get_line(25).context("Failed to get GPIO PIN 25")?;

    output
        .request(LineRequestFlags::OUTPUT, on as u8, "blinky")
        .map(|_| ())
        .context("Failed to set GPIO PIN 25")
}
