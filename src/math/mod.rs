use serde::Deserialize;
use std::fmt::{Display, Formatter};

pub mod kinematics;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    x: f64,
    y: f64,
}

impl From<CarPosition> for Point {
    fn from(value: CarPosition) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarPosition {
    x: f64,
    y: f64,
    angle: f64, // radians
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarTwist {
    delta_x: f64,
    delta_y: f64,
    delta_theta: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment(pub Point, pub Point);

impl Segment {
    pub fn new(p1: Point, p2: Point) -> Segment {
        Segment(p1, p2)
    }

    pub fn get_length(&self) -> f64 {
        let dx = self.1.x - self.0.x;
        let dy = self.1.y - self.0.y;
        dx.hypot(dy)
    }

    pub fn get_slope(&self) -> f64 {
        let dx = self.1.x - self.0.x;
        let dy = self.1.y - self.0.y;
        dy.atan2(dx)
    }

    pub fn get_midpoint(&self) -> Point {
        Point {
            x: (self.0.x + self.1.x) / 2.0,
            y: (self.0.y + self.1.y) / 2.0,
        }
    }

    pub fn get_point_at_distance(&self, distance: f64) -> Point {
        let Segment(p1, p2) = self;
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let length = (dx * dx + dy * dy).sqrt();
        let ratio = distance / length;
        Point {
            x: p1.x + dx * ratio,
            y: p1.y + dy * ratio,
        }
    }

    pub fn get_distance_to_point(&self, point: Point) -> f64 {
        let Segment(p1, p2) = self;
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let length = (dx * dx + dy * dy).sqrt();
        let ratio = ((point.x - p1.x) * dx + (point.y - p1.y) * dy) / (length * length);
        let closest_point = Point {
            x: p1.x + dx * ratio,
            y: p1.y + dy * ratio,
        };
        let dx = point.x - closest_point.x;
        let dy = point.y - closest_point.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/**
 * wrapping in interval [-PI, PI]
 */
fn angle_wrap(radians: f64) -> f64{
    const PI: f64 = std::f64::consts::PI;
    let mut new_angle = (radians + PI) % (2.0 * PI);

    if new_angle < 0.0 {
        new_angle += 2.0 * PI
    }

    new_angle - PI
}

fun Radians.angleWrap(): Radians {
var newAngle = (value + Math.PI) % (2.0 * Math.PI)

if (newAngle < 0.0)
newAngle += 2.0 * Math.PI

return Radians(newAngle - Math.PI)
}
