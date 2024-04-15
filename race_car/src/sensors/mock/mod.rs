use crate::sensors::manager::SensorManager;
use crate::sensors::{BasicSensor, GpsCoordinates, SensorData, SensorName};

pub fn add_all_sensors(sensor_manager: &mut SensorManager) {
    sensor_manager.add_sensor(MockImuSensor);
    sensor_manager.add_sensor(MockUltrasonicSensor);
    sensor_manager.add_sensor(MockGps);
    sensor_manager.add_sensor(MockVelocitySensor);
}

struct MockImuSensor;

impl BasicSensor for MockImuSensor {
    fn name(&self) -> SensorName {
        SensorName::Imu
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Imu {
            quaternion: rand::random::<[f32; 4]>(),
            acceleration: rand::random::<[f32; 3]>(),
        }
    }
}

struct MockGps;

impl BasicSensor for MockGps {
    fn name(&self) -> SensorName {
        SensorName::Gps
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Gps(GpsCoordinates {
            x: rand::random(),
            y: rand::random(),
            z: rand::random(),
            confidence: rand::random::<u8>() % 100,
        })
    }
}

struct MockUltrasonicSensor;

impl BasicSensor for MockUltrasonicSensor {
    fn name(&self) -> SensorName {
        SensorName::Ultrasonic
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Ultrasonic(rand::random::<f32>() * 10_f32)
    }
}

struct MockVelocitySensor;

impl BasicSensor for MockVelocitySensor {
    fn name(&self) -> SensorName {
        SensorName::Velocity
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Velocity(rand::random())
    }
}
