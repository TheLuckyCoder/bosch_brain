use crate::actuators::ActuatorName;

/// Trait defining the configuration and behavior of a motor/actuator driver.
///
/// The `ActuatorInput` generic parameter represents the type of actuator input
/// produced by `command_to_actuator_input`. For example, it could be `DutyCycle`,
/// a voltage, or another control type.
pub trait ActuatorConfig<ActuatorInput>: Sized + Send + 'static {
    /// Converts a normalized motor command into an actuator input.
    ///
    /// The `command` is expected to be in the range [-1.0, 1.0].
    /// If `command` is outside this range, the result is clamped.
    ///
    /// # Parameters
    /// - `command`: Normalized input command.
    ///
    /// # Returns
    /// - `ActuatorInput`: The actuator input corresponding to the normalized command.
    fn command_to_actuator_input(&self, command: f64) -> ActuatorInput;

    /// Serializes the configuration into a JSON string.
    fn get_config_json(&self) -> String;

    /// Generates an HTML representation of the configuration for a given actuator.
    fn get_config_html(&self, name: ActuatorName) -> String;

    /// Parses a configuration from a JSON string.
    fn parse_config(config: &str) -> Result<Self, serde_json::Error>;
}
