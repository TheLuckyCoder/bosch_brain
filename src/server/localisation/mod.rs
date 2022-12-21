use std::str;

use rsa::pkcs8::DecodePublicKey;
use rsa::RsaPublicKey;
use tokio::net::UdpSocket;

pub use data::*;

mod data;
mod position_listener;
mod verify_data;

const CAR_ID: i32 = 7; // TODO How would I know?

fn parse_port(buffer: &[u8]) -> Option<u16> {
    str::from_utf8(buffer).ok()?.parse::<u16>().ok()
}

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

async fn verify_server(port: u16) -> std::io::Result<()> {
    let socket = UdpSocket::bind(format!("0.0.0.0:{port}")).await?;

    let mut message_buffer = [0u8; 4096];
    let mut signature_buffer = [0u8; 4096];
    let message_size = socket.recv(&mut message_buffer).await?;
    let signature_size = socket.recv(&mut signature_buffer).await?;

    let public_key = RsaPublicKey::from_public_key_pem(include_str!("publickey_server.pem"))
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Could not parse public key",
            )
        })?;

    if message_size == 0
        || signature_size == 0
        || !verify_data::verify(
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

pub async fn run_localization(on_receive_data: fn(RobotPos)) -> std::io::Result<()> {
    // First receive the port to listen on
    let port = listen_for_port().await?;

    // Verify the server authentication and acknowledge connection
    verify_server(port).await?;

    // Listen for robot position
    position_listener::run_listener(port, on_receive_data).await?;

    Ok(())
}
