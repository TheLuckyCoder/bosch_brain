use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use tracing::error;

use crate::server::data::MovingObstaclePos;
use crate::server::ServerData;

async fn parse_data(socket: &UdpSocket) -> std::io::Result<MovingObstaclePos> {
    let mut buffer = [0; 4096];
    let size = socket.recv(&mut buffer).await?;
    let obstacle: MovingObstaclePos = serde_json::from_slice(&buffer[..size])?;
    Ok(obstacle)
}

pub async fn run_listener(sender: Sender<ServerData>) -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:50009").await?;

    loop {
        match parse_data(&socket).await {
            Ok(obstacle) => sender
                .send(ServerData::MovingObstacle(obstacle))
                .await
                .unwrap(),
            Err(e) => error!("Error occurred while receiving/parsing data: {}", e),
        }
    }
}
