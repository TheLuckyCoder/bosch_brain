use crate::serial::camera::LanesAngle;
use crate::server::data::{MovingObstaclePos, ServerCarPos, TrafficLightsStatus};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BrainData {
    pub car_pos: ServerCarPos,
    pub traffic_lights: TrafficLightsStatus,
    pub moving_obstacle: Option<MovingObstaclePos>,
    pub lanes_angle: LanesAngle,
}
