use crate::server::data::{MovingObstaclePos, ServerCarPos, TrafficLightsStatus};
use std::sync::mpsc::Receiver;
use tokio::task;

pub mod data;
mod environment;
mod localisation;
mod moving_obstacle;
mod traffic_lights;
mod utils;

pub enum ServerData {
    Localisation(ServerCarPos),
    TrafficLights(TrafficLightsStatus),
    MovingObstacle(MovingObstaclePos),
}

pub fn run_server_listeners() -> Receiver<ServerData> {
    let (tx, rx) = std::sync::mpsc::channel();

    let tx_clone = tx.clone();
    task::spawn(localisation::run_listener(move |robot_pos| {
        log::info!("Robot Pos: {:?}", robot_pos);
        tx_clone.send(ServerData::Localisation(robot_pos)).unwrap();
    }));

    let tx_clone = tx.clone();
    task::spawn(traffic_lights::run_listener(move |traffic_lights| {
        log::info!("Traffic Lights: {:?}", traffic_lights);
        tx_clone
            .send(ServerData::TrafficLights(traffic_lights))
            .unwrap();
    }));

    task::spawn(moving_obstacle::run_listener(move |moving_obstacle| {
        log::info!("MovingObstacle: {}", moving_obstacle);
        tx.send(ServerData::MovingObstacle(moving_obstacle))
            .unwrap();
    }));

    // task::spawn(environment::run_sender());

    rx
}
