use crate::server::data::{TrafficLight, TrafficLightsStatus};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use crate::server::ServerData;

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

pub async fn run_listener(sender: Sender<ServerData>) -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:50007").await?;

    let mut traffic_lights = TrafficLightsStatus::default();

    loop {
        match parse_data(&socket).await {
            Ok(traffic_light) => match traffic_light.id {
                1 => traffic_lights.0 = traffic_light.color,
                2 => traffic_lights.1 = traffic_light.color,
                3 => traffic_lights.2 = traffic_light.color,
                4 => traffic_lights.3 = traffic_light.color,
                _ => unreachable!(),
            },
            Err(e) => log::error!("Error occurred while receiving/parsing data: {}", e),
        }
        
        sender.send(ServerData::TrafficLights(traffic_lights)).await.unwrap();
    }
}
