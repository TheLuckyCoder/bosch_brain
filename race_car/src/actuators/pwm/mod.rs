pub mod mock;
pub mod pca9685;

#[derive(Debug, Default, Clone, Copy)]
pub struct Percentage {
    value: f64,
}

impl From<f64> for Percentage {
    fn from(value: f64) -> Self {
        Self {
            value: value.clamp(0.0, 100.0),
        }
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

pub trait Pwm: Send + 'static {
    fn set_duty_cycle(&mut self, percentage: Percentage);

    fn turn_off(&mut self);
}
