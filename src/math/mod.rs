use std::f64::consts::PI;
use std::ops::{Add, Mul};

pub use angle_wrap::*;
pub use circle::*;
pub use point::*;
pub use segment::*;
pub use almost_equals::*;

mod angle_wrap;
mod circle;
pub mod kinematics;
pub mod pid;
mod point;
mod segment;
mod almost_equals;

impl From<&CarPosition> for Point {
    fn from(value: &CarPosition) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CarPosition {
    pub x: f64,
    pub y: f64,
    pub angle: f64, // radians
}

impl CarPosition {
    pub fn new(x: f64, y: f64, angle: f64) -> Self {
        Self { x, y, angle }
    }

    pub fn rotate(&self, angle: f64) -> Self {
        let (x, y) = rotate_vector(self.x, self.y, angle - PI / 2.0);
        Self {
            x,
            y,
            angle: self.angle,
        }
    }
}

impl Add<&CarPosition> for CarPosition {
    type Output = CarPosition;

    fn add(self, rhs: &CarPosition) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            angle: self.angle + rhs.angle,
        }
    }
}

impl Mul<f64> for CarPosition {
    type Output = CarPosition;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
            angle: self.angle * rhs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Car {
    pub position: CarPosition,
    pub speed: f64,
}

impl Car {
    pub fn new(x: f64, y: f64, angle: f64, speed: f64) -> Self {
        let position = CarPosition::new(x, y, angle);
        Self { position, speed }
    }
}

pub fn rotate_vector(x: f64, y: f64, angle: f64) -> (f64, f64) {
    let cos = angle.cos();
    let sin = angle.sin();
    (x * cos - y * sin, y * cos + x * sin)
}
