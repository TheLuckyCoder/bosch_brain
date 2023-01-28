use crate::server::data::{TrafficLight, TrafficLightColor};
use tokio::net::UdpSocket;

async fn parse_data(socket: &UdpSocket) -> std::io::Result<TrafficLight> {
    let mut buffer = [0; 4096];
    let size = socket.recv(&mut buffer).await?;
    let traffic_light: TrafficLight = serde_json::from_slice(&buffer[..size])?;
    if traffic_light.id == 0 || traffic_light.id > 4 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Invalid traffic light id ({})", traffic_light.id),
        ));
    }
    Ok(traffic_light)
}

pub async fn run_listener(on_receive_data: impl Fn(Vec<TrafficLight>)) -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:50007").await?;

    let mut traffic_lights = [
        TrafficLight {
            id: 1,
            color: TrafficLightColor::Red,
        },
        TrafficLight {
            id: 2,
            color: TrafficLightColor::Red,
        },
        TrafficLight {
            id: 3,
            color: TrafficLightColor::Red,
        },
        TrafficLight {
            id: 4,
            color: TrafficLightColor::Red,
        },
    ];

    loop {
        match parse_data(&socket).await {
            Ok(traffic_light) => {
                traffic_lights[traffic_light.id as usize - 1] = traffic_light;
            }
            Err(e) => log::error!("Error occurred while receiving/parsing data: {}", e),
        }

        on_receive_data(traffic_lights.to_vec());
    }
}
