use std::{fs, io};

use crate::actuators::ActuatorName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub mock_sensors: bool,
    pub joystick: JoystickConfig,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct JoystickConfig {
    pub size: u8,
    pub opacity: u8,
    pub x_axis: ActuatorName,
    pub y_axis: ActuatorName,
}

impl Default for JoystickConfig {
    fn default() -> Self {
        Self {
            size: 13,
            opacity: 75,
            x_axis: ActuatorName::SpeedMotor,
            y_axis: ActuatorName::SteeringMotor,
        }
    }
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
