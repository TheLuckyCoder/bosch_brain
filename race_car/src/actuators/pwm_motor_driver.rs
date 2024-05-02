use crate::actuators::pwm::{Percentage, Pwm};
use crate::actuators::{Actuator, ActuatorName};
use serde::Deserialize;
use serde_json::Value;
use tracing::error;

pub trait PwmMotorDriverParams: Sized + Send + 'static {
    fn value_to_percentage(&self, value: f64) -> Percentage;

    fn get_config_html(&self, name: ActuatorName) -> String;

    fn parse(value: Value) -> Result<Self, serde_json::Error>;
}

pub struct PwmMotorDriver<PWM: Pwm, Params: PwmMotorDriverParams> {
    pwm: PWM,
    params: Params,
    last_value: f64,
    actuator_name: ActuatorName,
    paused: bool,
    inverse_direction: bool,
}

impl<PWM: Pwm, Params: PwmMotorDriverParams> PwmMotorDriver<PWM, Params> {
    pub fn new(
        actuator_name: ActuatorName,
        pwm: PWM,
        params: Params,
        inverse_direction: bool,
    ) -> Self {
        Self {
            pwm,
            actuator_name,
            params,
            last_value: f64::INFINITY,
            paused: false,
            inverse_direction,
        }
    }
}

impl<PWM: Pwm, Params: PwmMotorDriverParams> Actuator for PwmMotorDriver<PWM, Params> {
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

    fn get_config_html(&self) -> Option<String> {
        Some(self.params.get_config_html(self.actuator_name))
    }

    fn save_config(&mut self, data: Value) {
        // let content = data.to_string();
        let new_params = match Params::parse(data) {
            Ok(params) => params,
            Err(e) => {
                error!("Failed to parse params: {e}");
                return;
            }
        };

        self.params = new_params;
    }
}

impl<PWM: Pwm, Params: PwmMotorDriverParams> Drop for PwmMotorDriver<PWM, Params> {
    fn drop(&mut self) {
        self.stop()
    }
}
