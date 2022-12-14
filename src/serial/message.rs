#[derive(Debug, Clone)]
pub struct Message(String);

impl Message {
    pub fn get_string(&self) -> &String {
        &self.0
    }

    pub fn get_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Creates a command that sets the speed of the vehicle
    ///
    /// ```
    /// use crate::serial::Message;
    ///
    /// let msg = Message::speed(0.5_f32);
    /// assert_eq!(msg.get_string(), "#1:0.50;;\r\n".to_string());
    /// ```
    pub fn speed(velocity: f32) -> Message {
        Message(format!("#1:{:.2};;\r\n", velocity))
    }

    pub fn steer(angle: f32) -> Message {
        Message(format!("#2:{:.2};;\r\n", angle))
    }

    pub fn brake(angle: f32) -> Message {
        Message(format!("#3:{:.2};;\r\n", angle))
    }

    pub fn enable_pid(enable: bool) -> Message {
        Message(format!("#4:{};;\r\n", enable as u8))
    }

    pub fn enable_encoder_publisher(enable: bool) -> Message {
        Message(format!("#5:{};;\r\n", enable as u8))
    }

    pub fn pid_constants(k_p: f32, k_i: f32, k_d: f32, k_f: f32) -> Message {
        Message(format!(
            "#6:{:.5};{:.5};{:.5};{:.5};;\r\n",
            k_p, k_i, k_d, k_f
        ))
    }

    // pub fn no_command() -> Message {
    //     Message(format!("#7;;\r\n"))
    // }
}
