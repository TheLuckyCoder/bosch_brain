pub use data::*;
pub use imu::*;
pub use manager::*;
pub use motor_driver::*;
pub use ultrasonic::*;

mod data;
mod gps;
mod imu;
mod manager;
mod motor_driver;
mod ultrasonic;

pub trait BasicSensor {
    fn name(&self) -> &'static str;

    fn read_data(&mut self) -> SensorData;

    fn read_data_timed(&mut self) -> TimedSensorData {
        TimedSensorData::new(self.read_data())
    }
}
