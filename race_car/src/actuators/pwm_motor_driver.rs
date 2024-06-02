use std::io::Write;

use crate::actuators::{ActuatorDriver, ActuatorName};
use crate::actuators::pwm::{Percentage, PwmDriver};
use crate::utils::files::get_car_file;

pub trait PwmMotorDriverParams: Sized + Send + 'static {
    fn value_to_percentage(&self, value: f64) -> Percentage;

    fn get_config_json(&self) -> String;

    fn get_config_html(&self, name: ActuatorName) -> String;

    fn parse_config(config: &str) -> Result<Self, serde_json::Error>;
}

pub struct PwmMotorDriver<PWM: PwmDriver, Params: PwmMotorDriverParams> {
    pwm: PWM,
    params: Params,
    last_value: f64,
    actuator_name: ActuatorName,
    params_file_path: std::path::PathBuf,
    paused: bool,
    inverse_direction: bool,
}

impl<PWM: PwmDriver, Params: PwmMotorDriverParams> PwmMotorDriver<PWM, Params> {
    pub fn new(actuator_name: ActuatorName, pwm: PWM, default_params: Params) -> Self {
        let params_file_path = get_car_file(format!("motor_params_{actuator_name:?}.json"));
        let params = Self::read_params(params_file_path.as_path()).unwrap_or(default_params);

        Self {
            pwm,
            actuator_name,
            params,
            last_value: f64::INFINITY,
            params_file_path,
            paused: false,
            inverse_direction: false,
        }
    }

    pub fn set_inverse_direction(&mut self, inverse_direction: bool) {
        self.inverse_direction = inverse_direction;
    }

    fn read_params(params_file_path: &std::path::Path) -> Option<Params> {
        let read_params = std::fs::read_to_string(&params_file_path).ok()?;
        Params::parse_config(read_params.as_str()).ok()
    }
}

impl<PWM: PwmDriver, Params: PwmMotorDriverParams> ActuatorDriver for PwmMotorDriver<PWM, Params> {
    fn name(&self) -> ActuatorName {
        self.actuator_name
    }

    fn set_value(&mut self, value: f64) {
        // Steering motor should turn right when given a positive value
        let input = value.clamp(-1.0, 1.0) * if self.inverse_direction { -1.0 } else { 1.0 };

        if self.paused || (input - self.last_value).abs() < 10e-6 {
            return;
        }

        self.last_value = input;

        let duty_cycle = self.params.value_to_percentage(value);

        self.pwm.set_duty_cycle(duty_cycle);
    }

    fn stop(&mut self) {
        self.pwm.turn_off();
    }

    fn pause(&mut self) {
        self.paused = true;
        self.stop();
    }

    fn resume(&mut self) {
        self.paused = false;
    }

    fn is_paused(&self) -> bool {
        self.paused
    }

    fn get_config_json(&self) -> Option<String> {
        Some(self.params.get_config_json())
    }

    fn get_config_html(&self) -> Option<String> {
        Some(self.params.get_config_html(self.actuator_name))
    }

    fn save_config(&mut self, config: String) -> std::io::Result<()> {
        let new_params = match Params::parse_config(config.as_str()) {
            Ok(params) => params,
            Err(e) => return Err(std::io::Error::other(e)),
        };

        std::fs::File::create_new(self.params_file_path.as_path())?.write_all(config.as_bytes())?;

        self.params = new_params;

        Ok(())
    }
}

impl<PWM: PwmDriver, Params: PwmMotorDriverParams> Drop for PwmMotorDriver<PWM, Params> {
    fn drop(&mut self) {
        self.stop()
    }
}
