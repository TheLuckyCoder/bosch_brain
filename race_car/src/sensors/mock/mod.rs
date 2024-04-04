use mint::{Quaternion, Vector3};
use crate::sensors::{BasicSensor, GpsCoordinates, ImuData, SensorData, SensorName};

pub struct MockImuSensor;

impl BasicSensor for MockImuSensor {
    fn name(&self) -> SensorName {
        SensorName::Imu
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Imu(ImuData {
            quaternion: Quaternion::from(rand::random::<[f32; 4]>()),
            acceleration: Vector3::from(rand::random::<[f32; 3]>()),
        })
    }
}

pub struct MockGps;

impl BasicSensor for MockGps {
    fn name(&self) -> SensorName {
       SensorName::Gps
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Gps(GpsCoordinates {
            x: rand::random(),
            y: rand::random(),
            z: rand::random(),
            confidence: rand::random(),
        })
    }
}

pub struct MockUltrasonicSensor;

impl BasicSensor for MockUltrasonicSensor {
    fn name(&self) -> SensorName {
        SensorName::Ultrasonic
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Distance(rand::random())
    }
}

pub struct MockVelocitySensor;

impl BasicSensor for MockVelocitySensor {
    fn name(&self) -> SensorName {
        SensorName::Velocity
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Velocity(rand::random())
    }
}
