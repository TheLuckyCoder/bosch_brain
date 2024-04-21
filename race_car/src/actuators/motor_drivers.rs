use crate::actuators::pwm::Percentage;
use crate::actuators::pwm_motor_driver::PwmMotorDriverParams;
use crate::actuators::ActuatorName;
use askama::Template;

pub struct VelocityMotorParams {
    pub min: f64,
    pub lower_middle: f64,
    pub upper_middle: f64,
    pub max: f64,
}

impl PwmMotorDriverParams for VelocityMotorParams {
    fn value_to_percentage(&self, value: f64) -> Percentage {
        // Maps an input number that is between -1 and 1 (float) to a percentage than can't be smaller than percentage_minimum and bigger than percentage_maximum
        // If the input is smaller than -1 or bigger than 1 it gives equivalent to it (percentage_minimum/maximum)
        let percentage = if value != 0.0 {
            if value > 0.0 {
                self.upper_middle + value * (self.max - self.upper_middle)
            } else {
                self.lower_middle + -value * (self.min - self.lower_middle)
            }
        } else {
            (self.lower_middle + self.upper_middle) / 2.0
        };

        percentage.into()
    }

    fn get_config_html(&self, name: ActuatorName) -> String {
        #[derive(Template)]
        #[template(path = "components/motor_config.html")]
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
}

pub struct SteeringMotorParams {
    pub min: f64,
    pub middle: f64,
    pub max: f64,
}

impl PwmMotorDriverParams for SteeringMotorParams {
    fn value_to_percentage(&self, value: f64) -> Percentage {
        // Maps an input number that is between -1 and 1 (float) to a percentage than can't be smaller than percentage_minimum and bigger than percentage_maximum
        // If the input is smaller than -1 or bigger than 1 it gives equivalent to it (percentage_minimum/maximum)
        let percentage = if value != 0.0 {
            if value > 0.0 {
                self.middle + value * (self.max - self.middle)
            } else {
                self.middle + -value * (self.min - self.middle)
            }
        } else {
            self.middle
        };

        percentage.into()
    }

    fn get_config_html(&self, name: ActuatorName) -> String {
        String::from("Hello")
    }
}
