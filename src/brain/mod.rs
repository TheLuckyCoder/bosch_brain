use crate::serial::camera::{get_camera_data_receiver, CameraData, LanesAngle};
use crate::server::data::{MovingObstaclePos, ServerCarPos, TrafficLightsStatus};
use crate::server::{run_server_listeners, ServerData};
use std::sync::{Arc, Mutex};
use tokio::task;

struct BrainData {
    car_pos: ServerCarPos,
    traffic_lights: TrafficLightsStatus,
    moving_obstacle: Option<MovingObstaclePos>,
    lanes_angle: LanesAngle,
}

impl BrainData {
    fn default() -> Self {
        Self {
            car_pos: ServerCarPos::default(),
            traffic_lights: TrafficLightsStatus::default(),
            moving_obstacle: None,
            lanes_angle: LanesAngle::default(),
        }
    }
}

pub fn brain() {
    let brain_data = Arc::new(Mutex::new(BrainData::default()));

    update_data_from_server(brain_data.clone());
    update_data_from_camera(brain_data);
}

fn update_data_from_server(brain_data: Arc<Mutex<BrainData>>) {
    // Receive data from the server and update the brain data
    task::spawn_blocking(move || {
        let rx = run_server_listeners();

        while let Ok(server_data) = rx.recv() {
            let mut data = brain_data.lock().unwrap();

            match server_data {
                ServerData::Localisation(car_pos) => data.car_pos = car_pos,
                ServerData::TrafficLights(traffic_lights) => data.traffic_lights = traffic_lights,
                ServerData::MovingObstacle(moving_obstacle) => {
                    data.moving_obstacle = Some(moving_obstacle)
                }
            }
        }
    });
}

fn update_data_from_camera(brain_data: Arc<Mutex<BrainData>>) {
    let camera_receiver = get_camera_data_receiver();

    task::spawn_blocking(move || {
        while let Ok(camera_data) = camera_receiver.recv() {
            let mut data = brain_data.lock().unwrap();

            match camera_data {
                CameraData::LanesAngle(lanes_angle) => data.lanes_angle = lanes_angle,
            }
        }
    });
}
