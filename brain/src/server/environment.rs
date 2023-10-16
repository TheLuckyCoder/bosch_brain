use std::thread::sleep;
use std::time::Duration;

use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
use rsa::{RsaPrivateKey, RsaPublicKey};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Receiver;

use crate::server::data::EnvironmentalObstacle;
use crate::server::utils::{check_authentication, listen_for_port, sign_message, CAR_ID};

async fn establish_server_connection(server_address: &String) -> std::io::Result<()> {
    let public_key = RsaPublicKey::from_public_key_pem(include_str!("publickey_server.pem"))
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Could not parse public key",
            )
        })?;
    let private_key = RsaPrivateKey::from_pkcs8_pem(include_str!("privatekey_client.pem"))
        .map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Could not parse private key",
            )
        })?;

    let socket = UdpSocket::bind(&server_address).await?;

    // Send car id
    let message = CAR_ID.as_bytes();
    let signature = sign_message(message, private_key)?;
    socket.send(message).await?;
    sleep(Duration::from_millis(100));
    socket.send(signature.as_slice()).await?;

    check_authentication(public_key, socket).await?;
    log::info!("Connected to server address {server_address}");

    Ok(())
}

async fn send_data_to_environment_server(
    server_address: &String,
    mut rx: Receiver<EnvironmentalObstacle>,
) -> std::io::Result<()> {
    let server_socket = UdpSocket::bind(server_address).await?;

    while let Some(environmental_obstacle) = rx.recv().await {
        let serialized_obstacle = serde_json::to_string(&environmental_obstacle)?;
        server_socket.send(serialized_obstacle.as_ref()).await?;
        sleep(Duration::from_millis(100));
    }

    Ok(())
}

pub async fn run_sender(rx: Receiver<EnvironmentalObstacle>) -> std::io::Result<()> {
    let server_address = listen_for_port("0.0.0.0:25565").await?;

    // Verify the server authentication and acknowledge connection
    establish_server_connection(&server_address).await?;

    send_data_to_environment_server(&server_address, rx).await?;

    Ok(())
}
