use std::sync::Mutex;

pub use distance::*;
pub use imu::*;
pub use motor_driver::*;

mod distance;
mod imu;
mod motor_driver;

#[derive(Debug)]
pub enum SensorData {
    Distance(Option<f32>),
    Acceleration(mint::Vector3<f32>),
    Gyroscope(mint::Quaternion<f32>),
}

pub struct SensorManager {
    imu: Option<Mutex<GenericImu>>,
    distance_sensor: Option<Mutex<DistanceSensor>>,
}

impl SensorManager {
    pub fn new() -> Self {
        Self {
            imu: GenericImu::new()
                .map(|imu| Mutex::new(imu))
                .map_err(|e| log::error!("Generic IMU failed to initialize: {e}"))
                .ok(),
            // TODO Don't hardcode temperature
            distance_sensor: DistanceSensor::new(21f32)
                .map(|sensor| Mutex::new(sensor))
                .map_err(|e| log::error!("Distance Sensor failed to initialize: {e}"))
                .ok(),
            // TODO Camera
        }
    }

    pub fn check_imu(&self) -> bool {
        self.imu.is_some()
    }

    pub fn check_distance_sensor(&self) -> bool {
        self.distance_sensor.is_some()
    }

    pub fn check_camera(&self) -> bool {
        false
    }

    pub fn imu(&self) -> &Mutex<GenericImu> {
        self.imu.as_ref().unwrap()
    }

    pub fn distance_sensor(&self) -> &Mutex<DistanceSensor> {
        self.distance_sensor.as_ref().unwrap()
    }

    // pub fn get_sensor_data(&mut self) -> Result<Receiver<SensorData>, String> {
    //     let (tx, rx) = crossbeam_channel::unbounded();

    // std::thread::spawn(move || loop {
    //     std::thread::sleep(Duration::from_millis(100));
    //
    //     tx.send(SensorData::Distance(distance_sensor.get_distance_cm()))
    //         .unwrap();
    //     tx.send(SensorData::Acceleration(imu.get_acceleration()))
    //         .unwrap();
    //     tx.send(SensorData::Gyroscope(imu.get_quaternion()))
    //         .unwrap();
    // });

    // Ok(rx)
    // }
}
