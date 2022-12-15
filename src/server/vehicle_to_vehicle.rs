use tokio::net::UdpSocket;

use crate::server::data::VehicleToVehicle;

async fn parse_data(socket: &UdpSocket) -> std::io::Result<VehicleToVehicle> {
    let mut buffer = [0; 4096];
    let size = socket.recv(&mut buffer).await?;
    let vehicle: VehicleToVehicle = serde_json::from_slice(&buffer[..size])?;
    Ok(vehicle)
}

pub async fn run_listener(on_receive_data: fn(VehicleToVehicle)) -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:50009").await?;

    loop {
        match parse_data(&socket).await {
            Ok(vehicle) => on_receive_data(vehicle),
            Err(e) => {
                log::error!("Error occurred while receiving/parsing data: {}", e);
            }
        }
    }
}
