use std::f64::consts::PI;
use std::ops::Add;

pub mod kinematics;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: Point) -> f64 {
        let x = self.x - other.x;
        let y = self.y - other.y;
        x.hypot(y)
    }
}

impl From<CarPosition> for Point {
    fn from(value: CarPosition) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
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

impl Add<CarTwist> for CarPosition {
    type Output = CarPosition;

    fn add(self, rhs: CarTwist) -> Self::Output {
        Self::Output {
            x: self.x + rhs.delta_x,
            y: self.y + rhs.delta_y,
            angle: self.angle + rhs.delta_theta,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CarTwist {
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_theta: f64,
}

impl CarTwist {
    pub fn new(delta_x: f64, delta_y: f64, delta_theta: f64) -> Self {
        Self {
            delta_x,
            delta_y,
            delta_theta,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment(pub Point, pub Point);

impl Segment {
    pub fn new(p1: Point, p2: Point) -> Self {
        Self(p1, p2)
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
 * Wrapping in interval [-PI, PI]
 */
pub fn angle_wrap(radians: f64) -> f64 {
    let mut new_angle = (radians + PI) % (2.0 * PI);

    if new_angle < 0.0 {
        new_angle += 2.0 * PI
    }

    new_angle - PI
}

/**
 * automatically figures out if the shortest distance between two angles on the trigonometric circle is left or right
 * and returns this newly computed angle
 */
pub fn shortcut_wrap(radians: f64) -> f64 {
    if radians.abs() < (2.0 * PI + radians).abs() {
        angle_wrap(radians)
    } else {
        angle_wrap(PI * 2.0 + radians)
    }
}
