use serde::Deserialize;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MovingObstacle {
    id: i32,
    timestamp: i64,
    #[serde(rename = "coor")]
    position: (f32, f32),
    #[serde(rename = "rot")]
    angle: (f32, f32),
}

impl Display for MovingObstacle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MovingObstacle {{ id: {}, timestamp: {}, position: {:?}, angle: {:?} }}",
            self.id, self.timestamp, self.position, self.angle
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum TrafficLightColor {
    Red = 0,
    Yellow = 1,
    Green = 2,
}

impl Display for TrafficLightColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TrafficLightColor::Red => write!(f, "Red"),
            TrafficLightColor::Yellow => write!(f, "Yellow"),
            TrafficLightColor::Green => write!(f, "Green"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct TrafficLight {
    pub id: u8,
    #[serde(rename = "state")]
    pub color: TrafficLightColor,
}

impl Display for TrafficLight {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TrafficLight {{ id: {}, color: {} }}",
            self.id, self.color
        )
    }
}
