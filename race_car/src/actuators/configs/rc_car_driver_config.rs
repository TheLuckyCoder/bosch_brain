use crate::actuators::configs::actuator_config::ActuatorConfig;
use crate::actuators::ActuatorName;
use askama::Template;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use crate::actuators::configs::actuator_input_types::DutyCycle;

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct EscMotorConfig {
    /// Represents the duty cycle percentages for PWM control of a motor:
    /// - `min`: Full speed in the negative direction.
    /// - `(min, lower_middle)`: Speed range in the negative direction; below `lower_middle`, the motor is off.
    /// - `(lower_middle, upper_middle)`: Motor is off; no movement.
    /// - `(upper_middle, max)`: Speed range in the positive direction; above `upper_middle`, the motor is off.
    /// - `max`: Full speed in the positive direction.
    /// Some motors may have a very small off range or even a single value for the off state.
    #[serde_as(as = "DisplayFromStr")]
    pub min: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub lower_middle: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub upper_middle: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub max: f64,
}

impl ActuatorConfig<DutyCycle> for EscMotorConfig {
    fn command_to_actuator_input(&self, command: f64) -> DutyCycle {
        /// Converts a value in the range of -1 to 1 (inclusive) into a percentage.
        ///
        /// The output percentage is constrained to be no less than `min` and no greater than `max`.
        /// If the input value is outside the range of -1 to 1, it will be mapped to the corresponding
        /// `min` or `max` value.
        ///
        /// - If the input value is positive, it calculates the percentage based on the upper range.
        /// - If the input value is negative, it calculates the percentage based on the lower range.
        /// - If the input value is zero, it returns the average of `lower_middle` and `upper_middle`.

        let norm_command = command.clamp(-1.0, 1.0);

        let duty_cycle = if norm_command != 0.0 {
            if norm_command > 0.0 {
                self.upper_middle + norm_command * (self.max - self.upper_middle)
            } else {
                self.lower_middle + -norm_command * (self.min - self.lower_middle)
            }
        } else {
            (self.lower_middle + self.upper_middle) / 2.0
        };

        duty_cycle.into()
    }

    fn get_config_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("Serialization should not fail")
    }

    fn get_config_html(&self, name: ActuatorName) -> String {
        #[derive(Template)]
        #[template(path = "components/velocity_motor_config.html")]
        struct ConfigTemplate {
            name: ActuatorName,
            min: f64,
            lower_middle: f64,
            upper_middle: f64,
            max: f64,
        }

        let template = ConfigTemplate {
            name,
            min: self.min,
            lower_middle: self.lower_middle,
            upper_middle: self.upper_middle,
            max: self.max,
        };

        template
            .render()
            .unwrap_or_else(|e| format!("Failed to render config: {e}"))
    }

    fn parse_config(config: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(config)
    }
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct SteeringMotorConfig {
    #[serde_as(as = "DisplayFromStr")]
    pub min: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub middle: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub max: f64,
}

impl ActuatorConfig<DutyCycle> for SteeringMotorConfig {
    fn command_to_actuator_input(&self, command: f64) -> DutyCycle {
        let norm_command = command.clamp(-1.0, 1.0);

        let duty_cycle = if norm_command != 0.0 {
            if norm_command > 0.0 {
                self.middle + norm_command * (self.max - self.middle)
            } else {
                self.middle + -norm_command * (self.min - self.middle)
            }
        } else {
            self.middle
        };

        duty_cycle.into()
    }

    fn get_config_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("Serialization should not fail")
    }

    fn get_config_html(&self, name: ActuatorName) -> String {
        #[derive(Template)]
        #[template(path = "components/steering_motor_config.html")]
        struct ConfigTemplate {
            name: ActuatorName,
            min: f64,
            middle: f64,
            max: f64,
        }

        let template = ConfigTemplate {
            name,
            min: self.min,
            middle: self.middle,
            max: self.max,
        };

        template
            .render()
            .unwrap_or_else(|e| format!("Failed to render config: {e}"))
    }

    fn parse_config(config: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(config)
    }
}
