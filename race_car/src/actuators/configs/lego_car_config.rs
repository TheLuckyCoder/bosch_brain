use askama::Template;
use crate::actuators::ActuatorName;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use crate::actuators::configs::actuator_input_types::{DualChannelDuty, DutyCycle};
use crate::actuators::configs::actuator_config::ActuatorConfig;

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
pub struct GeekServoConfig {
    /// Physical minimum rotation angle of the servo (degrees).
    #[serde_as(as = "DisplayFromStr")]
    pub physical_min_angle: f64,

    /// Physical maximum rotation angle of the servo (degrees).
    #[serde_as(as = "DisplayFromStr")]
    pub physical_max_angle: f64,

    /// Offset added to the nominal center position (degrees).
    #[serde_as(as = "DisplayFromStr")]
    pub angle_offset: f64,

    /// Minimum control angle relative to the offset (degrees).
    #[serde_as(as = "DisplayFromStr")]
    pub control_min_angle: f64,

    /// Maximum control angle relative to the offset (degrees).
    #[serde_as(as = "DisplayFromStr")]
    pub control_max_angle: f64,

    /// Minimum PWM pulse width corresponding to `physical_min_angle` (µs).
    #[serde_as(as = "DisplayFromStr")]
    pub servo_min_pulse: f64,

    /// Maximum PWM pulse width corresponding to `physical_max_angle` (µs).
    #[serde_as(as = "DisplayFromStr")]
    pub servo_max_pulse: f64,

    /// PWM frequency for the servo signal (Hz), e.g., 50 Hz.
    #[serde_as(as = "DisplayFromStr")]
    pub servo_freq_hz: f64,
}

fn map_range(x: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    (x - in_min) / (in_max - in_min) * (out_max - out_min) + out_min
}
impl ActuatorConfig<DutyCycle> for GeekServoConfig {
    fn command_to_actuator_input(&self, command: f64) -> DutyCycle {
        let norm_command = command.clamp(-1.0, 1.0);

        // reverse direction
        let v = -norm_command;

        // Map control value to physical angle
        let phys_angle = if v >= 0.0 {
            map_range(v, 0.0, 1.0, self.angle_offset, self.control_max_angle)
        } else {
            map_range(v, -1.0, 0.0, self.control_min_angle, self.angle_offset)
        }.clamp(self.physical_min_angle, self.physical_max_angle);
        // Theoretically, clamp is not necessary, but just in case control_min_angle and control_max_angle are outside the physical range.

        // Map angle to pulse width
        let pulse = map_range(
            phys_angle,
            self.physical_min_angle,
            self.physical_max_angle,
            self.servo_min_pulse,
            self.servo_max_pulse,
        );

        // Convert pulse to duty cycle percentage
        let period_us = 1_000_000.0 / self.servo_freq_hz;
        let duty_cycle = (pulse / period_us) * 100.0;
        duty_cycle.into()
    }

    fn get_config_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    fn get_config_html(&self, name: ActuatorName) -> String {
        #[derive(Template)]
        #[template(path = "components/pyzero_servo_config.html")]
        struct ConfigTemplate {
            name: ActuatorName,
            physical_min_angle: f64,
            physical_max_angle: f64,
            angle_offset: f64,
            control_min_angle: f64,
            control_max_angle: f64,
            servo_min_pulse: f64,
            servo_max_pulse: f64,
            servo_freq_hz: f64,
        }

        let template = ConfigTemplate {
            name,
            physical_min_angle: self.physical_min_angle,
            physical_max_angle: self.physical_max_angle,
            angle_offset: self.angle_offset,
            control_min_angle: self.control_min_angle,
            control_max_angle: self.control_max_angle,
            servo_min_pulse: self.servo_min_pulse,
            servo_max_pulse: self.servo_max_pulse,
            servo_freq_hz: self.servo_freq_hz,
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
#[derive(Serialize, Deserialize, Clone)]
pub struct LegoDcMotorConfig {
    /// Input supply voltage (V)
    #[serde_as(as = "DisplayFromStr")]
    pub supply_voltage: f64,

    /// Target max motor voltage (V)
    /// the motor driver seems to be outputting a bit more, for ex for 9 it did 10.23
    #[serde_as(as = "DisplayFromStr")]
    pub target_max_voltage: f64,

    /// PWM resolution (bits)
    #[serde_as(as = "DisplayFromStr")]
    pub pwm_resolution: u8,

    /// PWM frequency (Hz)
    #[serde_as(as = "DisplayFromStr")]
    pub pwm_freq_hz: f64,
}

impl ActuatorConfig<DualChannelDuty> for LegoDcMotorConfig {
    fn command_to_actuator_input(&self, command: f64) -> DualChannelDuty {
        let norm_command = command.clamp(-1.0, 1.0);

        // Map normalized command to scaled PWM value
        let max_duty = (1 << self.pwm_resolution) - 1;
        let scaled_duty = (norm_command.abs() * self.target_max_voltage / self.supply_voltage) * (max_duty as f64);

        let duty_percent = (scaled_duty / max_duty as f64) * 100.0;

        let (channel_a, channel_b) = if norm_command >= 0.0 {
            (DutyCycle::from(duty_percent), DutyCycle::zero())
        } else {
            (DutyCycle::zero(), DutyCycle::from(duty_percent))
        };

        DualChannelDuty {
            channel_a,
            channel_b,
        }
    }

    fn get_config_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    fn get_config_html(&self, name: ActuatorName) -> String {
        #[derive(Template)]
        #[template(path = "components/pyzero_motor_config.html")]
        struct ConfigTemplate {
            name: ActuatorName,
            target_max_voltage: f64,
            supply_voltage: f64,
            pwm_resolution: u8,
            pwm_freq_hz: f64,
        }

        let template = ConfigTemplate {
            name,
            target_max_voltage: self.target_max_voltage,
            supply_voltage: self.supply_voltage,
            pwm_resolution: self.pwm_resolution,
            pwm_freq_hz: self.pwm_freq_hz,
        };

        template
            .render()
            .unwrap_or_else(|e| format!("Failed to render config: {e}"))
    }

    fn parse_config(config: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(config)
    }
}

impl ActuatorConfig<DutyCycle> for LegoDcMotorConfig {
    fn command_to_actuator_input(&self, command: f64) -> DutyCycle {
        let norm_command = command.clamp(-1.0, 1.0);

        // Map normalized command to scaled PWM value
        let max_duty = (1 << self.pwm_resolution) - 1;
        let scaled_duty = (norm_command.abs() * self.target_max_voltage / self.supply_voltage) * (max_duty as f64);

        let duty_percent = (scaled_duty / max_duty as f64) * 100.0;

        let sign = if norm_command >= 0.0 { 1 } else { -1 };

        DutyCycle {
            magnitude: duty_percent.clamp(0.0, 100.0) as u32,
            sign,
        }
    }

    fn get_config_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    fn get_config_html(&self, name: ActuatorName) -> String {
        #[derive(Template)]
        #[template(path = "components/pyzero_motor_config.html")]
        struct ConfigTemplate {
            name: ActuatorName,
            target_max_voltage: f64,
            supply_voltage: f64,
            pwm_resolution: u8,
            pwm_freq_hz: f64,
        }

        let template = ConfigTemplate {
            name,
            target_max_voltage: self.target_max_voltage,
            supply_voltage: self.supply_voltage,
            pwm_resolution: self.pwm_resolution,
            pwm_freq_hz: self.pwm_freq_hz,
        };

        template
            .render()
            .unwrap_or_else(|e| format!("Failed to render config: {e}"))
    }

    fn parse_config(config: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(config)
    }
}
