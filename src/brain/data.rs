use crate::math::Car;
use crate::serial::camera::{get_camera_data_receiver, CameraData, LanesAngle, Signs};
use crate::server::data::{MovingObstaclePos, ServerCarPos, TrafficLightsStatus};
use crate::server::{run_server_listeners, ServerData};
use sensors::{get_sensor_data, SensorData};
use std::sync::{Arc, Mutex};
use tokio::task;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BrainData {
    pub car: Car,
    pub server_pos: ServerCarPos,
    pub traffic_lights: TrafficLightsStatus,
    pub moving_obstacle: Option<MovingObstaclePos>,
    pub lanes_angle: LanesAngle,
    pub signs: Signs,
    pub distance_sensor: Option<f32>,
}

pub fn update_data(brain_data: Arc<Mutex<BrainData>>) {
    update_server_data(brain_data.clone());
    update_sensor_data(brain_data.clone());
    update_camera_data(brain_data);
}

fn update_server_data(brain_data: Arc<Mutex<BrainData>>) {
    // Receive data from the server and update the brain data
    task::spawn(async move {
        let mut rx = run_server_listeners();

        while let Some(server_data) = rx.recv().await {
            dbg!(&server_data);
            let mut data = brain_data.lock().unwrap();

            match server_data {
                ServerData::CarPos(car_pos) => data.server_pos = car_pos,
                ServerData::TrafficLights(traffic_lights) => data.traffic_lights = traffic_lights,
                ServerData::MovingObstacle(moving_obstacle) => {
                    data.moving_obstacle = Some(moving_obstacle)
                }
            }
        }
    });
}

fn update_sensor_data(brain_data: Arc<Mutex<BrainData>>) {
    let sensors_receiver = get_sensor_data().expect("Failed to initialize sensors data");

    std::thread::spawn(move || {
        while let Ok(sensors) = sensors_receiver.recv() {
            let mut data = brain_data.lock().unwrap();

            println!("Sensor Data: {sensors:?}");

            match sensors {
                SensorData::Distance(distance) => data.distance_sensor = distance,
                SensorData::Acceleration(_) => {}
                SensorData::Gyroscope(_) => {}
            }
        }
    });
}

fn update_camera_data(brain_data: Arc<Mutex<BrainData>>) {
    let camera_receiver = get_camera_data_receiver();

    std::thread::spawn(move || {
        while let Ok(camera_data) = camera_receiver.recv() {
            let mut data = brain_data.lock().unwrap();

            match camera_data {
                CameraData::LanesAngle(lanes_angle) => data.lanes_angle = lanes_angle,
                CameraData::Signs(_) => {}
            }
        }
    });
}
