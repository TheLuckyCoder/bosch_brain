use std::f64::consts::PI;
use std::fmt::Display;
use std::ops::{Add, Mul};

pub use angle_wrap::*;
pub use circle::*;
pub use point::*;
pub use segment::*;

mod angle_wrap;
mod circle;
pub mod kinematics;
pub mod pid;
mod point;
mod segment;

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
    pub heading_angle: f64, // radians
}

impl CarPosition {
    pub fn new(x: f64, y: f64, angle: f64) -> Self {
        Self { x, y, heading_angle: angle }
    }

    pub fn rotate(&self, angle: f64) -> Self {
        let (x, y) = rotate_vector(self.x, self.y, angle - PI / 2.0);
        Self {
            x,
            y,
            heading_angle: self.heading_angle,
        }
    }
}

impl Display for CarPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.3}, {:.3}, {:.3})", self.x, self.y, self.heading_angle)
    }
}

impl Add<&CarPosition> for CarPosition {
    type Output = CarPosition;

    fn add(self, rhs: &CarPosition) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            heading_angle: self.heading_angle + rhs.heading_angle,
        }
    }
}

impl Mul<f64> for CarPosition {
    type Output = CarPosition;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
            heading_angle: self.heading_angle * rhs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Car {
    pub position: CarPosition,
    pub speed: f64,
    pub steering_angle: f64, // radians
}

impl Car {
    pub fn new(x: f64, y: f64, angle: f64, speed: f64, steering_angle: f64) -> Self {
        let position = CarPosition::new(x, y, angle);
        Self { position, speed, steering_angle }
    }
}

impl Display for Car {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Car {{\n x: {:.3},\n y: {:.3},\n heading_angle: {:.3} ,\n speed: {:.3},\n steering_angle: {:.3} }}",
               self.position.x, self.position.y, self.position.heading_angle.to_degrees(), self.speed, self.steering_angle.to_degrees())
    }
}

pub fn rotate_vector(x: f64, y: f64, angle: f64) -> (f64, f64) {
    let cos = angle.cos();
    let sin = angle.sin();
    (x * cos - y * sin, y * cos + x * sin)
}
