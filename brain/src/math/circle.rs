use crate::math::Point;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    pub center: Point,
    pub radius: f64,
}

impl Circle {
    pub fn new(center: Point, radius: f64) -> Self {
        Self { center, radius }
    }

    pub fn find_center(p1: Point, p2: Point, p3: Point) -> Circle {
        let a = p1.x - p3.x;
        let b = p1.y - p3.y;
        let c = (p1.x * p1.x - p3.x * p3.x) + (p1.y * p1.y - p3.y * p3.y);
        let d = p2.x - p3.x;
        let e = p2.y - p3.y;
        let f = (p2.x * p2.x - p3.x * p3.x) + (p2.y * p2.y - p3.y * p3.y);

        let center = Point::new(
            (b * f - e * c) / (2.0 * b * d - 2.0 * a * e),
            (d * c - a * f) / (2.0 * b * d - 2.0 * a * e),
        );

        Circle::new(
            center,
            ((center.x - p1.x) * (center.x - p1.x) + (center.y - p1.y) * (center.y - p1.y)).sqrt(),
        )
    }
}

impl Display for Circle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Circle {{ center: {}, radius: {} }}",
            self.center, self.radius
        )
    }
}
