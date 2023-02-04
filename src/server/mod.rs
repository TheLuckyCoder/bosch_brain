use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task;

use crate::server::data::{
    EnvironmentalObstacle, MovingObstaclePos, ServerCarPos, TrafficLightsStatus,
};

pub mod data;
mod environment;
mod localisation;
mod moving_obstacle;
mod traffic_lights;
mod utils;

#[derive(Debug)]
pub enum ServerData {
    CarPos(ServerCarPos),
    TrafficLights(TrafficLightsStatus),
    MovingObstacle(MovingObstaclePos),
}

pub fn run_server_listeners() -> Receiver<ServerData> {
    let (tx, rx) = mpsc::channel(64);

    task::spawn(localisation::run_listener(tx.clone()));
    task::spawn(traffic_lights::run_listener(tx.clone()));
    task::spawn(moving_obstacle::run_listener(tx));

    rx
}

pub fn environment_server_publisher() -> Sender<EnvironmentalObstacle> {
    let (tx, rx) = mpsc::channel(64);

    task::spawn(environment::run_sender(rx));

    tx
}
