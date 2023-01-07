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

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct ServerCarPos {
    pub x: f32,
    pub y: f32,
}

impl Display for ServerCarPos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ServerCarPos {{ x: {:.5}, y: {:.5} }}", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ObstacleDescritpion {
    TS_Stop = 1,
    TS_Priority = 2,
    TS_Parking = 3,
    TS_Crosswalk = 4,
    TS_HighwayEntrance = 5,
    TS_HighwayExit = 6,
    TS_Roundabout = 7,
    TS_OneWayRoad = 8,
    TrafficLight = 9,
    StaticCarOnRoad = 10,
    StaticCarOnParking = 11,
    PedestrianOnCrosswalk = 12,
    PedestrianOnRoad = 13,
    Roadblock = 14,
    BumpyRoad = 15,
}

impl Display for ObstacleDescritpion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ObstacleDescritpion::TS_Stop => write!(f, "TS_Stop"),
            ObstacleDescritpion::TS_Priority => write!(f, "TS_Priority"),
            ObstacleDescritpion::TS_Parking => write!(f, "TS_Parking"),
            ObstacleDescritpion::TS_Crosswalk => write!(f, "TS_Crosswalk"),
            ObstacleDescritpion::TS_HighwayEntrance => write!(f, "TS_HighwayEntrance"),
            ObstacleDescritpion::TS_HighwayExit => write!(f, "TS_HighwayExit"),
            ObstacleDescritpion::TS_Roundabout => write!(f, "TS_Roundabout"),
            ObstacleDescritpion::TS_OneWayRoad => write!(f, "TS_OneWayRoad"),
            ObstacleDescritpion::TrafficLight => write!(f, "TrafficLight"),
            ObstacleDescritpion::StaticCarOnRoad => write!(f, "StaticCarOnRoad"),
            ObstacleDescritpion::StaticCarOnParking => write!(f, "StaticCarOnParking"),
            ObstacleDescritpion::PedestrianOnCrosswalk => write!(f, "PedestrianOnCrosswalk"),
            ObstacleDescritpion::PedestrianOnRoad => write!(f, "PedestrianOnRoad"),
            ObstacleDescritpion::Roadblock => write!(f, "Roadblock"),
            ObstacleDescritpion::BumpyRoad => write!(f, "BumpyRoad"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct EnvironmentalObstacle {
    pub id: i8,
    #[serde(rename = "coor")]
    pub position: (f32, f32),
    pub desc: ObstacleDescritpion,
}

impl Display for EnvironmentalObstacle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EnvironmentalObstacle {{ id: {}, position: {:?}, description: {} }}",
            self.id, self.position, self.desc
        )
    }
}
