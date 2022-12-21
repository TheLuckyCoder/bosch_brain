use tokio::net::UdpSocket;

use crate::server::localisation::RobotPos;

async fn parse_data(socket: &UdpSocket) -> std::io::Result<RobotPos> {
    let mut buffer = [0; 4096];
    let size = socket.recv(&mut buffer).await?;
    let obstacle: RobotPos = serde_json::from_slice(&buffer[..size])?;
    Ok(obstacle)
}

pub async fn run_listener(port: u16, on_receive_data: fn(RobotPos)) -> std::io::Result<()> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{port}")).await?;

    loop {
        match parse_data(&socket).await {
            Ok(obstacle) => on_receive_data(obstacle),
            Err(e) => log::error!("Error occurred while receiving/parsing data: {}", e),
        }
    }
}
