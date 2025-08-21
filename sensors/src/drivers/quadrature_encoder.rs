use std::thread::spawn;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context;
use linux_embedded_hal::gpio_cdev::{Chip, LineRequestFlags};
use tokio::time::sleep;

use crate::{HardwareSensor, SensorData, name::SensorName};

/// Read a two-channel (A/B) optical encoder as a *signed* rotational speed source (MotorRPS).
///
/// This implementation uses X4 decoding (counts on every transition). Set
/// `COUNTS_PER_ROTATION = slots_per_rotation * 4.0` for a classic square slot disk.
pub struct QuadratureEncoderSensor {
    total_counts: Arc<AtomicIsize>, // monotonic signed count (increments/decrements)
    last_sampled_counts: isize,     // last snapshot to compute deltas
    last_time: Instant,
    chip: Chip,                     // keep chip alive for lifetime of line handles
}

impl QuadratureEncoderSensor {
    /// Create a new quadrature sensor using GPIO lines `a_line` and `b_line` on `gpio_chip_path`.
    /// `COUNTS_PER_ROTATION` should reflect X4 decoding (typically slots * 4.0).
    pub fn new(gpio_chip_path: &str, a_line: u32, b_line: u32) -> anyhow::Result<Self> {
        let mut chip = Chip::new(gpio_chip_path).context("Failed to open GPIO chip")?;

        let a = chip
            .get_line(a_line)
            .context("Failed to get line A")?
            .request(LineRequestFlags::INPUT | LineRequestFlags::ACTIVE_LOW, 0, "quad_enc_A")
            .context("Failed to request line A")?;

        let b = chip
            .get_line(b_line)
            .context("Failed to get line B")?
            .request(LineRequestFlags::INPUT | LineRequestFlags::ACTIVE_LOW, 0, "quad_enc_B")
            .context("Failed to request line B")?;

        let total_counts = Arc::new(AtomicIsize::new(0));
        {
            let counts = total_counts.clone();
            tokio::spawn(async move {
                // Quadrature state machine: previous (row) -> current (col)
                // +1 for forward, -1 for reverse, 0 for no/invalid transition
                const LUT: [[i8; 4]; 4] = [
                    [ 0,  1, -1,  0], // from 00
                    [-1,  0,  0,  1], // from 01
                    [ 1,  0,  0, -1], // from 10
                    [ 0, -1,  1,  0], // from 11
                ];

                let mut prev = ((a.get_value().unwrap_or(0) != 0) as u8) << 1
                    | ((b.get_value().unwrap_or(0) != 0) as u8);

                loop {
                    let a_now = (a.get_value().unwrap_or(0) != 0) as u8;
                    let b_now = (b.get_value().unwrap_or(0) != 0) as u8;
                    let curr = (a_now << 1) | b_now;

                    if curr != prev {
                        // apply LUT; ignore illegal 2-bit jumps (treated as 0)
                        let step = LUT[prev as usize][curr as usize] as isize;
                        if step != 0 {
                            counts.fetch_add(step, Ordering::Relaxed);
                        }
                        prev = curr;
                    }

                    // ~1 kHz polling. You can increase if your slots are dense / speed is high.
                    sleep(Duration::from_millis(1)).await;
                }
            });
        }

        Ok(Self {
            total_counts,
            last_sampled_counts: 0,
            last_time: Instant::now(),
            chip,
        })
    }

    /// Signed shaft speed in revolutions per second (RPS).
    fn calculate_signed_rps(&mut self) -> f64 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_time).as_secs_f64();
        self.last_time = now;

        let total = self.total_counts.load(Ordering::Relaxed);
        let delta_counts = total - self.last_sampled_counts;
        self.last_sampled_counts = total;

        if elapsed <= f64::EPSILON {
            return 0.0;
        }

        // Convert counts in window -> revolutions in window (X4 decoding expected)
        let revs = (delta_counts as f64) / COUNTS_PER_ROTATION;
        revs / elapsed // signed RPS
    }
}

/// With X4 decoding, set this to: slots_per_rotation * 4.0
const COUNTS_PER_ROTATION: f64 = 80.0; // e.g., 20 slots → 80 counts/rev

impl HardwareSensor for QuadratureEncoderSensor {
    fn name(&self) -> SensorName {
        // Reuse an existing name to minimize downstream churn; adjust if you have a dedicated variant.
        SensorName::QuadratureEncoder
    }

    fn read_data(&mut self) -> SensorData {
        let rps = self.calculate_signed_rps();
        // Signed allowed via f64: negative = reverse
        SensorData::MotorRPS(rps)
    }

    fn read_debug(&mut self) -> String {
        format!(
            "total_counts={} (counts/rev={})",
            self.total_counts.load(Ordering::Relaxed),
            COUNTS_PER_ROTATION
        )
    }

    fn end_calibration(&mut self) -> anyhow::Result<()> { Ok(()) }
}