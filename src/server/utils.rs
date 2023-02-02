use std::str;

use rsa::{Pss, PublicKey, RsaPrivateKey, RsaPublicKey};
use tokio::net::UdpSocket;

pub const CAR_ID: &str = "69"; // TODO How would I know?

pub fn parse_port(buffer: &[u8]) -> Option<u16> {
    str::from_utf8(buffer).ok()?.parse::<u16>().ok()
}

pub fn sign_message(message: &[u8], private_key: RsaPrivateKey) -> std::io::Result<Vec<u8>> {
    let padding = Pss::new_with_salt::<md5::Md5>(48usize);
    private_key
        .sign(padding, message)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
}

fn verify_signature(public_key: RsaPublicKey, message: &[u8], signature: &[u8]) -> bool {
    let padding = Pss::new_with_salt::<md5::Md5>(48usize);
    public_key.verify(padding, message, signature).is_ok()
}

pub async fn check_authentication(
    public_key: RsaPublicKey,
    socket: UdpSocket,
) -> std::io::Result<()> {
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

    Ok(())
}

pub async fn listen_for_port<S: AsRef<str>>(address: S) -> std::io::Result<String> {
    let socket = UdpSocket::bind(address.as_ref()).await?;

    let mut buffer = [0u8; 1500];
    let (size, address) = socket.recv_from(&mut buffer).await?;
    let port = parse_port(&buffer[..size]).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Could not parse port from received data",
        )
    })?;

    Ok(format!("{}:{}", address.ip().to_string(), port))
}
