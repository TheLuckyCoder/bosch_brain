use std::{fs, io};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub mock_sensors: bool,
}

impl ServerConfig {
    pub fn read_server_config() -> io::Result<Self> {
        toml::from_str(fs::read_to_string("config.toml")?.as_ref()).map_err(io::Error::other)
    }

    pub fn save_to_file(&self) -> io::Result<()> {
        let string = toml::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write("config.toml", string)
    }
}
