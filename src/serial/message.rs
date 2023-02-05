use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Speed(f32),
    Steer(f32),
    Brake(f32),
    EnablePid(bool),
    EnableEncoderPublisher(bool),
    PidParams {
        k_p: f32,
        k_i: f32,
        k_d: f32,
        k_f: f32,
    },
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Message::Speed(speed) => write!(f, "#1:{speed:.2};;\r\n"),
            Message::Steer(angle) => write!(f, "#2:{angle:.2};;\r\n"),
            Message::Brake(angle) => write!(f, "#3:{angle:.2};;\r\n"),
            Message::EnablePid(enable) => write!(f, "#4:{};;\r\n", enable as u8),
            Message::EnableEncoderPublisher(enable) => write!(f, "#5:{};;\r\n", enable as u8),
            Message::PidParams { k_p, k_i, k_d, k_f } => {
                write!(f, "#6:{k_p:.5};{k_i:.5};{k_d:.5};{k_f:.5};;\r\n")
            }
        }
    }
}
