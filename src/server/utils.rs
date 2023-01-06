use rsa::{PaddingScheme, PublicKey, RsaPublicKey};
use std::str;
use tokio::net::UdpSocket;

pub fn parse_port(buffer: &[u8]) -> Option<u16> {
    str::from_utf8(buffer).ok()?.parse::<u16>().ok()
}

pub fn verify_signature(public_key: RsaPublicKey, message: &[u8], signature: &[u8]) -> bool {
    let padding = PaddingScheme::new_pss_with_salt::<md5::Md5>(48usize);
    public_key.verify(padding, message, signature).is_ok()
}

pub async fn listen_for_port() -> std::io::Result<String> {
    let socket = UdpSocket::bind("0.0.0.0:50009").await?;

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
