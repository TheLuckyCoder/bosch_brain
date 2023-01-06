use crate::server::utils::listen_for_port;

pub async fn run_environment() -> std::io::Result<()> {
    let server_address = listen_for_port().await?;

    Ok(())
}
