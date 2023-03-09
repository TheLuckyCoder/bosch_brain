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

pub fn angle_between_vectors(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dot = x1 * x2 + y1 * y2;
    let det = x1 * y2 - y1 * x2;
    det.atan2(dot)
}

pub fn angle_between_vector_and_x_axis(x: f64, y: f64) -> f64 {
    y.atan2(x)
}

pub fn get_heading_vector_and_angle(circle: Circle, current_position: Point, heading_vector_scale: f64) -> (Point, f64) {
    // Calculate vector from center to point
    let mut dx = current_position.x - circle.center.x;
    let mut dy = current_position.y - circle.center.y;

    let theta = dx.atan2(-dy); // Angle between Ox and tangential vector

    // Calculate tangential vector
    let tx = heading_vector_scale * (-dy + current_position.x);
    let ty = heading_vector_scale * (dx + current_position.y);

    (Point::new(tx, ty), theta)
}

pub fn get_heading_vector_and_angle_v2(circle: Circle, current_position: Point, next_waypoint: Point, heading_vector_length: f64) -> (Point, f64) {
    println!("\n Next waypoint: {:?}", next_waypoint);
    // Calculate vector from center to point
    let dx = current_position.x - circle.center.x;
    let dy = current_position.y - circle.center.y;

    let arc_lengths = arc_lengths(current_position.x, current_position.y,
                                  next_waypoint.x, next_waypoint.y, circle.radius);

    println!("Arc lengths: {:?}", arc_lengths);

    let clockwise = arc_lengths.0 > arc_lengths.1;
    println!("Clockwise: {}", clockwise);

    if clockwise {
        let dx = -dx;
        let dy = -dy;
    }

    let theta = dx.atan2(-dy); // Angle between Ox and tangential vector

    // Calculate tangential vector
    let tx = (-dy + current_position.x);
    let ty = (dx + current_position.y);

    println!("Heading vector: {:?}", Point::new(tx, ty));

    (Point::new(tx, ty), theta)
}

pub fn arc_length(x1: f64, y1: f64, x2: f64, y2: f64, radius: f64) -> f64 {
    let d = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    let angle_in_radians = 2.0 * (d / (2.0 * radius)).asin();
    let arc_length = radius * angle_in_radians;
    arc_length
}

pub fn arc_lengths(x1: f64, y1: f64, x2: f64, y2: f64, radius: f64) -> (f64, f64) {
    let d = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
    let angle_in_radians = 2.0 * (d / (2.0 * radius)).asin();
    let shorter_arc_length = radius * angle_in_radians;
    let longer_arc_length = 2.0 * PI * radius - shorter_arc_length;
    (shorter_arc_length, longer_arc_length) // if shorter_arc_length < longer_arc_length -> counterclockwise
}