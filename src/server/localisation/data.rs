use serde::Deserialize;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct RobotPos {
    pub x: f32,
    pub y: f32,
}

impl Display for RobotPos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RobotPos {{ x: {:.5}, y: {:.5} }}", self.x, self.y)
    }
}
