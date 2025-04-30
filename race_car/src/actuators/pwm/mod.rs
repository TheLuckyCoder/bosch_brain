pub mod mock;
pub mod pca9685;
pub mod lego_servo;
mod software_pwm;

#[derive(Debug, Clone, Copy)]
pub struct Percentage {
    value: f64,
    is_positive: bool,
}

impl Default for Percentage {
    fn default() -> Self {
        Self {
            value: 0.0,
            is_positive: true,
        }
    }
}

impl From<f64> for Percentage {
    fn from(value: f64) -> Self {
        if value < 0.0 {
            return Self {
                value: (-value).clamp(0.0, 100.0),
                is_positive: false,
            };
        }
        Self {
            value: value.clamp(0.0, 100.0),
            is_positive: true,
        }
    }
}

impl Percentage {
    pub fn zero() -> Self {
        Percentage::default()
    }
}

// impl TryFrom<f64> for Percentage {
//     type Error = ();
//     fn try_from(value: f64) -> Result<Self, Self::Error> {
//         if value < 0.0 || value > 100.0 {
//             return Err(());
//         }
//         Ok(Self { value })
//     }
// }

pub trait PwmDriver: Send + 'static {
    fn set_duty_cycle(&mut self, percentage: Percentage);

    fn turn_off(&mut self);
}
