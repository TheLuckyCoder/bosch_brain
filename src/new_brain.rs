use std::sync::mpsc::{channel, Sender};

use sensors::{DistanceSensor, GenericImu, MotorDriver};
use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug)]
pub enum State {
    Standby,
    Calibration,
    RemoteControl,
    AutonomousControl,
}

impl Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl State {
    pub fn from_str<T: AsRef<str>>(string: T) -> State {
        match string.as_ref().to_ascii_lowercase().as_str() {
            "remote" => State::RemoteControl,
            "auto" => State::AutonomousControl,
            _ => State::Standby,
        }
    }
}

pub fn start() -> Result<Sender<State>, String> {
    let (sender, receiver) = channel();

    let mut imu = GenericImu::new()?;
    let mut distance_sensor = DistanceSensor::new(imu.get_temperature()? as f32)?;
    let mut motor_diver = MotorDriver::new().unwrap();

    std::thread::spawn(move || {
        while let Ok(state) = receiver.recv() {
            match state {
                State::Standby => {}
                State::Calibration => {
                    if !imu.is_calibrated().unwrap() {
                        imu.start_calibration().unwrap();
                    }

                    distance_sensor.start_calibration();
                    motor_diver.start_calibration();
                }
                State::RemoteControl => {}
                State::AutonomousControl => {}
            }
        }

        log::error!("Channel closed");
    });

    Ok(sender)
}
