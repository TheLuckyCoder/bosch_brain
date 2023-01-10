use crate::math::Point;

#[derive(Debug, Clone, PartialEq)]
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
