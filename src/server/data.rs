use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MovingObstaclePos {
    id: i32,
    timestamp: i64,
    #[serde(rename = "coor")]
    position: (f32, f32),
    #[serde(rename = "rot")]
    angle: (f32, f32),
}

impl Display for MovingObstaclePos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MovingObstacle {{ id: {}, timestamp: {}, position: {:?}, angle: {:?} }}",
            self.id, self.timestamp, self.position, self.angle
        )
    }
}

// region Traffic Lights

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize_repr)]
#[repr(u8)]
pub enum TrafficLightColor {
    #[default]
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

#[derive(Debug, Deserialize)]
pub struct TrafficLight {
    pub id: u8,
    #[serde(rename = "state")]
    pub color: TrafficLightColor,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct TrafficLightsStatus(
    pub TrafficLightColor,
    pub TrafficLightColor,
    pub TrafficLightColor,
    pub TrafficLightColor,
);

impl Display for TrafficLightsStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TrafficLights {{ 1: {}, 2: {}, 3: {}, 4: {} }}",
            self.0, self.1, self.2, self.3
        )
    }
}

impl TrafficLightsStatus {
    fn as_slice(&self) -> [TrafficLightColor; 4] {
        [self.0, self.1, self.2, self.3]
    }
}

// endregion Traffic Lights

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct ServerCarPos {
    pub x: f32,
    pub y: f32,
}

impl Display for ServerCarPos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ServerCarPos {{ x: {:.5}, y: {:.5} }}", self.x, self.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ObstacleId {
    TsStop = 1,
    TsPriority = 2,
    TsParking = 3,
    TsCrosswalk = 4,
    TsHighwayEntrance = 5,
    TsHighwayExit = 6,
    TsRoundabout = 7,
    TsOneWayRoad = 8,
    TrafficLight = 9,
    StaticCarOnRoad = 10,
    StaticCarOnParking = 11,
    PedestrianOnCrosswalk = 12,
    PedestrianOnRoad = 13,
    Roadblock = 14,
    BumpyRoad = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentalObstacle {
    #[serde(rename = "OBS")]
    pub id: ObstacleId,
    pub x: f32,
    pub y: f32,
}

impl Display for EnvironmentalObstacle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EnvironmentalObstacle {{ id: {:?}, x: {}, y: {} }}",
            self.id, self.x, self.y
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let obstacle = EnvironmentalObstacle {
            id: ObstacleId::TsStop,
            x: 1.0,
            y: 2.0,
        };

        let serialized = serde_json::to_string(&obstacle).unwrap();
        assert_eq!(serialized, r#"{"OBS":1,"x":1.0,"y":2.0}"#);
    }
}
