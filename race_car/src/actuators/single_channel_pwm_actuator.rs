use std::io::Write;

use crate::actuators::{Actuator, ActuatorName};
use crate::actuators::configs::actuator_config::ActuatorConfig;
use crate::actuators::configs::actuator_input_types::DutyCycle;
use crate::actuators::drivers::PwmDriver;
use crate::utils::files::get_car_file;

pub struct SingleChannelPwmActuator<PWM: PwmDriver, Config: ActuatorConfig<DutyCycle>> {
    pwm: PWM,
    config: Config,
    last_command: f64,
    actuator_name: ActuatorName,
    params_file_path: std::path::PathBuf,
    paused: bool,
    reverse_direction: bool,
}

impl<PWMDriver: PwmDriver, Config: ActuatorConfig<DutyCycle>> SingleChannelPwmActuator<PWMDriver, Config> {
    pub fn new(actuator_name: ActuatorName, pwm: PWMDriver, default_config: Config) -> Self {
        let params_file_path = get_car_file(format!("motor_params_{actuator_name:?}.json"));
        let config = Self::read_params(params_file_path.as_path()).unwrap_or(default_config);

        Self {
            pwm,
            actuator_name,
            config,
            last_command: f64::INFINITY,
            params_file_path,
            paused: false,
            reverse_direction: false,
        }
    }

    pub fn invert_direction(&mut self) {
        self.reverse_direction = !self.reverse_direction;
    }

    fn read_params(params_file_path: &std::path::Path) -> Option<Config> {
        let read_params = std::fs::read_to_string(&params_file_path).ok()?;
        Config::parse_config(read_params.as_str()).ok()
    }
}

impl<PWM: PwmDriver, Config: ActuatorConfig<DutyCycle>> Actuator for SingleChannelPwmActuator<PWM, Config> {
    fn name(&self) -> ActuatorName {
        self.actuator_name
    }

    fn set_command(&mut self, command: f64) {
        // Steering motor should turn right when given a positive value
        let norm_command = command.clamp(-1.0, 1.0) * if self.reverse_direction { -1.0 } else { 1.0 };

        if self.paused || (norm_command - self.last_command).abs() < 10e-6 {
            return;
        }

        self.last_command = norm_command;

        let duty_cycle = self.config.command_to_actuator_input(command);

        self.pwm.set_duty_cycle(duty_cycle);
        println!("Setting {} to {:?}%", self.actuator_name, duty_cycle);
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

    fn get_config(&self) -> Option<String> {
        Some(self.config.get_config_json())
    }

    fn get_config_html(&self) -> Option<String> {
        Some(self.config.get_config_html(self.actuator_name))
    }

    fn save_config(&mut self, config: String) -> std::io::Result<()> {
        let new_params = match Config::parse_config(config.as_str()) {
            Ok(params) => params,
            Err(e) => return Err(std::io::Error::other(e)),
        };

        std::fs::File::create_new(self.params_file_path.as_path())?.write_all(config.as_bytes())?;

        self.config = new_params;

        Ok(())
    }
}

impl<PWMDriver: PwmDriver, Config: ActuatorConfig<DutyCycle>> Drop for SingleChannelPwmActuator<PWMDriver, Config> {
    fn drop(&mut self) {
        self.stop()
    }
}
