use rsa::pkcs8::DecodePublicKey;
use rsa::RsaPublicKey;
use tokio::net::UdpSocket;

use crate::server::data::ServerCarPos;
use crate::server::utils::{parse_port, verify_signature};

const CAR_ID: &str = "7"; // TODO How would I know?

async fn listen_for_port() -> std::io::Result<u16> {
    let socket = UdpSocket::bind("0.0.0.0:50009").await?;

    let mut buffer = [0u8; 1500];
    let size = socket.recv(&mut buffer).await?;
    let port = parse_port(&buffer[..size]).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Could not parse port from received data",
        )
    })?;

    Ok(port)
}

async fn establish_server_connection(port: u16) -> std::io::Result<()> {
    // Parse public key
    let public_key = RsaPublicKey::from_public_key_pem(include_str!("publickey_server.pem"))
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Could not parse public key",
            )
        })?;

    let socket = UdpSocket::bind(format!("0.0.0.0:{port}")).await?;

    // Send car id
    socket.send(CAR_ID.as_bytes()).await?;

    // Receive response
    let mut message_buffer = [0u8; 4096];
    let mut signature_buffer = [0u8; 4096];
    let message_size = socket.recv(&mut message_buffer).await?;
    let signature_size = socket.recv(&mut signature_buffer).await?;

    // Verify message and signature
    if message_size == 0
        || signature_size == 0
        || !verify_signature(
            public_key,
            &message_buffer[..message_size],
            &signature_buffer[..signature_size],
        )
    {
        socket.send(b"Authentication not ok").await?;
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Authentication not ok",
        ));
    }

    socket.send(b"Authentication ok").await?;
    log::info!("Connected to server port {port}");

    Ok(())
}

async fn parse_robot_position(socket: &UdpSocket) -> std::io::Result<ServerCarPos> {
    let mut buffer = [0; 4096];
    let size = socket.recv(&mut buffer).await?;
    let obstacle: ServerCarPos = serde_json::from_slice(&buffer[..size])?;
    Ok(obstacle)
}

async fn run_localisation_listener(
    port: u16,
    on_receive_data: fn(ServerCarPos),
) -> std::io::Result<()> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{port}")).await?;

    loop {
        match parse_robot_position(&socket).await {
            Ok(obstacle) => on_receive_data(obstacle),
            Err(e) => log::error!("Error occurred while receiving/parsing data: {}", e),
        }
    }
}

pub async fn run_localization(on_receive_data: fn(ServerCarPos)) -> std::io::Result<()> {
    // First receive the port to listen on
    let port = listen_for_port().await?;

    // Verify the server authentication and acknowledge connection
    establish_server_connection(port).await?;

    // Listen for robot position
    run_localisation_listener(port, on_receive_data).await?;

    Ok(())
}
