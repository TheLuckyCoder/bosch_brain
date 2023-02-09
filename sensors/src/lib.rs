use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

use distance::DistanceSensor;
use imu::{ImuSpecs, Vec3};

pub mod distance;
pub mod imu;

#[derive(Debug)]
pub enum SensorData {
    Distance(Option<f32>),
    Acceleration(Vec3),
    Gyroscope(Vec3),
}

pub fn get_sensor_data() -> Result<Receiver<SensorData>, String> {
    let (tx, rx) = channel();
    let mut imu = imu::get_imu()?;
    let mut distance_sensor = distance::get_distance_sensor()?;

    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(100));
        let distance_1 = distance_sensor.get_distance().unwrap();

        tx.send(SensorData::Distance(distance_1)).unwrap();
        tx.send(SensorData::Acceleration(imu.get_acceleration()))
            .unwrap();
        tx.send(SensorData::Gyroscope(imu.get_gyroscope())).unwrap();
    });

    Ok(rx)
}
