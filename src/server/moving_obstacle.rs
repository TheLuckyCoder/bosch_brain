use tokio::net::UdpSocket;

use crate::server::data::MovingObstacle;

async fn parse_data(socket: &UdpSocket) -> std::io::Result<MovingObstacle> {
    let mut buffer = [0; 4096];
    let size = socket.recv(&mut buffer).await?;
    let obstacle: MovingObstacle = serde_json::from_slice(&buffer[..size])?;
    Ok(obstacle)
}

pub async fn run_listener(on_receive_data: fn(MovingObstacle)) -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:50009").await?;

    loop {
        match parse_data(&socket).await {
            Ok(obstacle) => on_receive_data(obstacle),
            Err(e) => log::error!("Error occurred while receiving/parsing data: {}", e),
        }
    }
}
