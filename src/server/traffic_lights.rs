use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tokio::net::UdpSocket;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrafficLightState {
    Red = 0,
    Yellow = 1,
    Green = 2,
}

impl Display for TrafficLightState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrafficLightState::Red => write!(f, "Red"),
            TrafficLightState::Yellow => write!(f, "Yellow"),
            TrafficLightState::Green => write!(f, "Green"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrafficLight {
    id: u8,
    state: TrafficLightState,
}

impl Display for TrafficLight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TrafficLight {{ id: {}, state: {} }}",
            self.id, self.state
        )
    }
}

async fn parse_data(socket: &UdpSocket) -> std::io::Result<TrafficLight> {
    let mut buffer = [0; 512];
    let size = socket.recv(&mut buffer).await?;
    Ok(serde_json::from_slice(&buffer[..size])?)
}

pub async fn run_listener(on_receive_data: fn(Vec<TrafficLight>)) -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:50007").await?;

    socket.connect("0.0.0.0:4096").await?; // TODO test ports
    let mut traffic_lights = [
        TrafficLight {
            id: 1,
            state: TrafficLightState::Red,
        },
        TrafficLight {
            id: 2,
            state: TrafficLightState::Red,
        },
        TrafficLight {
            id: 3,
            state: TrafficLightState::Red,
        },
        TrafficLight {
            id: 4,
            state: TrafficLightState::Red,
        },
    ];

    loop {
        on_receive_data(traffic_lights.to_vec());
        match parse_data(&socket).await {
            Ok(traffic_light) => {
                traffic_lights[traffic_light.id as usize - 1] = traffic_light;
            }
            Err(e) => {
                log::error!("Error occurred while receiving/parsing data: {}", e);
            }
        }

        on_receive_data(traffic_lights.to_vec());
    }
}
