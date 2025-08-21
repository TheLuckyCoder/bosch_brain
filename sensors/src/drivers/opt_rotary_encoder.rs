use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context;
use linux_embedded_hal::gpio_cdev::{Chip, LineRequestFlags};
use tokio::time::sleep;

use crate::{HardwareSensor, SensorData, name::SensorName};

pub struct OpticalRotaryEncoderSensor {
    count: Arc<AtomicUsize>,
    last_time: Instant,
    chip: Chip, // keep the chip alive for the lifetime of the line handle
}

impl OpticalRotaryEncoderSensor {
    /// Create a new sensor on the given GPIO chip and line
    pub fn new(gpio_chip_path: &str, line: u32) -> anyhow::Result<Self> {
        let mut chip = Chip::new(gpio_chip_path).context("Failed to open GPIO chip")?;
        let input_line = chip
            .get_line(line)
            .context("Failed to get GPIO line")?
            .request(LineRequestFlags::INPUT | LineRequestFlags::ACTIVE_LOW, 0, "optical_velocity_sensor")
            .context("Failed to request line")?;

        let count = Arc::new(AtomicUsize::new(0));

        // Spawn polling loop to count beam interruptions (rising edge)
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
                    // ~1 kHz polling. Tune for your hardware latency vs. CPU budget.
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

    /// Calculate shaft speed in revolutions per second (RPS)
    fn calculate_shaft_rps(&mut self) -> f64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_time).as_secs_f64();
        self.last_time = now;

        // number of pulses since last read
        let pulses = self.count.swap(0, Ordering::Relaxed) as f64;

        if elapsed <= f64::EPSILON {
            return 0.0; // avoid divide-by-zero on very fast successive calls
        }

        // pulses -> shaft revolutions in the measurement window
        let shaft_revs = pulses / PULSES_PER_ROTATION;

        // revolutions per second
        shaft_revs / elapsed
    }
}

// Number of interruptions (holes/slots) per one full shaft rotation
const PULSES_PER_ROTATION: f64 = 20.0; // ← adjust to your encoder

impl HardwareSensor for OpticalRotaryEncoderSensor {
    fn name(&self) -> SensorName {
        // Keeping the same name to avoid downstream changes; update if you have a dedicated RPM variant
        SensorName::OpticalRotaryEncoder
    }

    fn read_data(&mut self) -> SensorData {
        let rpm = self.calculate_shaft_rps();
        SensorData::MotorRPS(rpm)
    }

    fn read_debug(&mut self) -> String {
        format!(
            "Pulses since last read: {} (PPR={})",
            self.count.load(Ordering::Relaxed),
            PULSES_PER_ROTATION
        )
    }

    fn end_calibration(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}