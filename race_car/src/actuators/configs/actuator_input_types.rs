/// Represents a single PWM duty cycle for single-channel actuators.
/// Range: 0..=100 (%)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DutyCycle {
    pub magnitude: u32, // 0..=100
}

impl From<f64> for DutyCycle {
    fn from(value: f64) -> Self {
        Self {
            magnitude: value.clamp(0.0, 100.0) as u32,
        }
    }
}

impl From<&DutyCycle> for f64 {
    fn from(d: &DutyCycle) -> Self {
        d.magnitude as f64
    }
}

impl From<&DutyCycle> for u32 {
    fn from(d: &DutyCycle) -> Self {
        d.magnitude
    }
}

impl Default for DutyCycle {
    fn default() -> Self {
        DutyCycle { magnitude: 0 }
    }
}

impl DutyCycle {
    pub fn zero() -> Self {
        DutyCycle::default()
    }

    pub fn as_fraction(&self) -> f64 {
        self.magnitude as f64 / 100.0
    }
}

/// Represents a dual-channel PWM duty cycle (e.g., for H-bridge motor control).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DualChannelDuty {
    pub channel_a: DutyCycle,
    pub channel_b: DutyCycle,
}

impl Default for DualChannelDuty {
    fn default() -> Self {
        DualChannelDuty {
            channel_a: DutyCycle::default(),
            channel_b: DutyCycle::default(),
        }
    }
}
