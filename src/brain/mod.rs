use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task;

use crate::math::pid::PidController;
use crate::serial;
pub use data::*;

use crate::serial::camera::{get_camera_data_receiver, CameraData};
use crate::serial::Message;
use crate::server::{run_server_listeners, ServerData};

mod data;

pub fn start_brain() {
    let brain_data = Arc::new(Mutex::new(BrainData::default()));

    update_data_from_server(brain_data.clone());
    update_data_from_camera(brain_data.clone());

    process_data(brain_data);
}

fn process_data(brain_data: Arc<Mutex<BrainData>>) {
    let mut last_data = BrainData::default();
    let mut lane_pid = PidController::new(1.0, 0.5, 0.01);

    loop {
        std::thread::sleep(Duration::from_millis(1));

        let data = brain_data.lock().unwrap().clone();
        if last_data == data {
            continue;
        }
        last_data = data.clone();

        let total_angle = data.lanes_angle.left + data.lanes_angle.right;
        let steering_angle = lane_pid.compute(total_angle);
        serial::send(Message::Steer(steering_angle as f32));
    }
}

fn update_data_from_server(brain_data: Arc<Mutex<BrainData>>) {
    // Receive data from the server and update the brain data
    task::spawn(async move {
        let mut rx = run_server_listeners();

        while let Some(server_data) = rx.recv().await {
            dbg!(&server_data);
            let mut data = brain_data.lock().unwrap();

            match server_data {
                ServerData::CarPos(car_pos) => data.car_pos = car_pos,
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

    std::thread::spawn(move || {
        while let Ok(camera_data) = camera_receiver.recv() {
            let mut data = brain_data.lock().unwrap();

            match camera_data {
                CameraData::LanesAngle(lanes_angle) => data.lanes_angle = lanes_angle,
            }
        }
    });
}
