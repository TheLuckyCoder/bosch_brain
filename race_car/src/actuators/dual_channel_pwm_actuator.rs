use std::io::Write;

use crate::actuators::{Actuator, ActuatorName};
use crate::actuators::configs::actuator_config::ActuatorConfig;
use crate::actuators::configs::actuator_input_types::{DualChannelDuty};
use crate::actuators::drivers::PwmDriver;
use crate::utils::files::get_car_file;

pub struct DualChannelPwmActuator<PWM1: PwmDriver, PWM2: PwmDriver, Config: ActuatorConfig<DualChannelDuty>> {
    pwm_a: PWM1,
    pwm_b: PWM2,
    config: Config,
    last_command: f64,
    actuator_name: ActuatorName,
    params_file_path: std::path::PathBuf,
    paused: bool,
    reverse_direction: bool,
}

impl<PWM1: PwmDriver, PWM2: PwmDriver, Config: ActuatorConfig<DualChannelDuty>> DualChannelPwmActuator<PWM1, PWM2, Config>
{
    pub fn new(
        actuator_name: ActuatorName,
        pwm_a: PWM1,
        pwm_b: PWM2,
        default_config: Config,
    ) -> Self {
        let params_file_path = get_car_file(format!("motor_params_{actuator_name:?}.json"));
        let config = Self::read_params(params_file_path.as_path()).unwrap_or(default_config);
        Self {
            pwm_a,
            pwm_b,
            config,
            last_command: 0.0,
            actuator_name,
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

impl<PWM1: PwmDriver, PWM2: PwmDriver, Config: ActuatorConfig<DualChannelDuty>> Actuator for DualChannelPwmActuator<PWM1, PWM2, Config>
{
    fn name(&self) -> ActuatorName {
        self.actuator_name
    }

    fn set_command(&mut self, command: f64) {
        let norm_command =
            command.clamp(-1.0, 1.0) * if self.reverse_direction { -1.0 } else { 1.0 };

        if self.paused || (norm_command - self.last_command).abs() < 10e-6 {
            return;
        }

        self.last_command = norm_command;

        let duty = self.config.command_to_actuator_input(command);

        // self.pwm_a.set_duty_cycle(duty.channel_a);
        self.pwm_b.set_duty_cycle(duty.channel_b);

        println!(
            "Setting {} to A: {:?}%, B: {:?}%",
            self.actuator_name, duty.channel_a, duty.channel_b
        );
    }

    fn stop(&mut self) {
        self.pwm_a.turn_off();
        self.pwm_b.turn_off();
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

impl<PWM1: PwmDriver, PWM2: PwmDriver, Config: ActuatorConfig<DualChannelDuty>> Drop for DualChannelPwmActuator<PWM1, PWM2, Config>
{
    fn drop(&mut self) {
        self.stop()
    }
}
