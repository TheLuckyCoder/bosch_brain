use std::f64::consts::PI;
use std::ops::{Add, Mul};

mod angle_wrap;
pub mod kinematics;
mod point;
mod segment;

pub use angle_wrap::*;
pub use point::*;
pub use segment::*;

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
}

impl Add<&CarTwist> for CarPosition {
    type Output = CarPosition;

    fn add(self, rhs: &CarTwist) -> Self::Output {
        Self::Output {
            x: self.x + rhs.delta_x,
            y: self.y + rhs.delta_y,
            angle: self.angle + rhs.delta_angle,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CarTwist {
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_angle: f64,
}

impl CarTwist {
    pub fn new(delta_x: f64, delta_y: f64, delta_theta: f64) -> Self {
        Self {
            delta_x,
            delta_y,
            delta_angle: delta_theta,
        }
    }

    pub fn rotate(&self, angle: f64) -> Self {
        let (delta_x, delta_y) = rotate_vector(self.delta_x, self.delta_y, angle - PI / 2.0);
        Self {
            delta_x,
            delta_y,
            delta_angle: self.delta_angle,
        }
    }
}

impl Mul<f64> for CarTwist {
    type Output = CarTwist;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::Output {
            delta_x: self.delta_x * rhs,
            delta_y: self.delta_y * rhs,
            delta_angle: self.delta_angle * rhs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CarSpeed {
    pub speed: f64,
    pub angle: f64,
}

pub fn rotate_vector(x: f64, y: f64, angle: f64) -> (f64, f64) {
    let cos = angle.cos();
    let sin = angle.sin();
    (x * cos - y * sin, x * sin + y * cos)
}
