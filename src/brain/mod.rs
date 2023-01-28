use crate::server::data::{MovingObstacle, ServerCarPos, TrafficLight};
use crate::server::{run_server_listeners, ServerData};
use std::sync::{Arc, Mutex};
use tokio::task;

struct BrainData {
    car_pos: ServerCarPos,
    traffic_lights: Vec<TrafficLight>,
    moving_obstacle: Option<MovingObstacle>,
}

impl BrainData {
    fn default() -> Self {
        Self {
            car_pos: ServerCarPos { x: 0.0, y: 0.0 },
            traffic_lights: vec![],
            moving_obstacle: None,
        }
    }
}

pub fn brain() {
    let brain_data = Arc::new(Mutex::new(BrainData::default()));

    let brain_data_clone = brain_data.clone();
    // Receive data from the server and update the brain data
    task::spawn_blocking(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        run_server_listeners(tx);

        while let Ok(server_data) = rx.recv() {
            let mut data = brain_data_clone.lock().unwrap();

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
