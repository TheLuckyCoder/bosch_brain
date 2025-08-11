use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context;
use linux_embedded_hal::gpio_cdev::{Chip, LineRequestFlags};
use tokio::time::sleep;

use crate::{HardwareSensor, SensorData, name::SensorName};

pub struct OpticalVelocitySensor {
    count: Arc<AtomicUsize>,
    last_time: Instant,
    chip: Chip,
}

impl OpticalVelocitySensor {
    /// Create a new VelocitySensor on the given GPIO chip and line
    pub fn new(gpio_chip_path: &str, line: u32) -> anyhow::Result<Self> {
        let mut chip = Chip::new(gpio_chip_path).context("Failed to open GPIO chip")?;
        let input_line = chip
            .get_line(line)
            .context("Failed to get GPIO line")?
            .request(LineRequestFlags::INPUT | LineRequestFlags::ACTIVE_LOW, 0, "optical_velocity_sensor")
            .context("Failed to request line")?;

        let count = Arc::new(AtomicUsize::new(0));

        // Spawn polling loop to count beam interruptions
        {
            let count_clone = count.clone();
            tokio::spawn(async move {
                let mut last_state = false;
                loop {
                    let current_state = input_line.get_value().unwrap_or(0) == 1;
                    if current_state && !last_state {
                        count_clone.fetch_add(1, Ordering::Relaxed);
                    }
                    last_state = current_state;
                    sleep(Duration::from_millis(1)).await;
                }
            });
        }

        Ok(Self {
            count,
            last_time: Instant::now(),
            chip,
        })
    }

    /// Calculate linear velocity (m/s) of the car
    fn calculate_velocity(&mut self) -> f64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_time).as_secs_f64();
        self.last_time = now;

        let pulses = self.count.swap(0, Ordering::Relaxed) as f64;
        // Convert pulses to shaft rotations
        let shaft_rotations = pulses / PULSES_PER_ROTATION;
        // Convert shaft rotations to wheel rotations via gear ratio
        let wheel_rotations = shaft_rotations / GEAR_RATIO;
        // Linear distance traveled by the car
        let distance = wheel_rotations * WHEEL_CIRCUMFERENCE;

        // Velocity in meters per second
        distance / elapsed
    }
}

// Number of interruptions (holes) per one full shaft rotation
const PULSES_PER_ROTATION: f64 = 20.0; // adjust to your encoder
// Gear ratio: motor shaft rotations per one wheel rotation
const GEAR_RATIO: f64 = 1.0; // adjust based on your drivetrain
// Circumference of the actual ground-contacting wheel, in meters
const WHEEL_CIRCUMFERENCE: f64 = 0.21; // car wheel circumference

impl HardwareSensor for OpticalVelocitySensor {
    fn name(&self) -> SensorName {
        SensorName::OpticalVelocity
    }

    fn read_data(&mut self) -> SensorData {
        let v = self.calculate_velocity();
        SensorData::OpticalVelocity(v)
    }

    fn read_debug(&mut self) -> String {
        format!("Velocity pulses counted: {}", self.count.load(Ordering::Relaxed))
    }

    fn end_calibration(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
