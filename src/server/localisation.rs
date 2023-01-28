use rsa::pkcs8::DecodePublicKey;
use rsa::RsaPublicKey;
use tokio::net::UdpSocket;

use crate::server::data::ServerCarPos;
use crate::server::utils::{check_authentication, listen_for_port, CAR_ID};

async fn establish_server_connection(server_address: &String) -> std::io::Result<()> {
    // Parse public key
    let public_key = RsaPublicKey::from_public_key_pem(include_str!("publickey_server.pem"))
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Could not parse public key",
            )
        })?;

    let socket = UdpSocket::bind(&server_address).await?;

    // Send car id
    socket.send(CAR_ID.as_bytes()).await?;

    check_authentication(public_key, socket).await?;
    log::info!("Connected to server address {server_address}");

    Ok(())
}

async fn parse_position(socket: &UdpSocket) -> std::io::Result<ServerCarPos> {
    let mut buffer = [0; 4096];
    let size = socket.recv(&mut buffer).await?;
    let obstacle: ServerCarPos = serde_json::from_slice(&buffer[..size])?;
    Ok(obstacle)
}

async fn run_localisation_listener(
    server_address: String,
    on_receive_data: impl Fn(ServerCarPos),
) -> std::io::Result<()> {
    let socket = UdpSocket::bind(server_address).await?;

    loop {
        match parse_position(&socket).await {
            Ok(obstacle) => on_receive_data(obstacle),
            Err(e) => log::error!("Error occurred while receiving/parsing data: {}", e),
        }
    }
}

pub async fn run_listener(on_receive_data: impl Fn(ServerCarPos)) -> std::io::Result<()> {
    // First receive the port to listen on
    let server_address = listen_for_port("0.0.0.0:50009").await?;

    // Verify the server authentication and acknowledge connection
    establish_server_connection(&server_address).await?;

    // Listen for robot position
    run_localisation_listener(server_address, on_receive_data).await?;

    Ok(())
}
