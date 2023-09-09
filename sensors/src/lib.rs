use std::sync::mpsc::{channel, Receiver};

pub mod distance;
pub mod imu;
pub mod motor_driver;

#[derive(Debug)]
pub enum SensorData {
    Distance(Option<f32>),
    Acceleration(mint::Vector3<f32>),
    Gyroscope(mint::Quaternion<f32>),
}

pub fn get_sensor_data() -> Result<Receiver<SensorData>, String> {
    let (tx, rx) = channel();
    /*let mut imu = imu::get_imu()?;
    let mut distance_sensor = distance::get_distance_sensor(20f32)?;

    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(100));

        tx.send(SensorData::Distance(distance_sensor.get_distance_cm()))
            .unwrap();
        tx.send(SensorData::Acceleration(imu.get_acceleration()))
            .unwrap();
        tx.send(SensorData::Gyroscope(imu.get_quaternion()))
            .unwrap();
    });*/

    Ok(rx)
}
