use crate::sensors::manager::SensorManager;
use sensors::name::SensorName;
use sensors::{HardwareSensor, GpsCoordinates, SensorData};

pub fn add_all_mock_sensors(sensor_manager: &mut SensorManager) {
    sensor_manager.add_sensor(MockImuSensor);
    sensor_manager.add_sensor(MockUltrasonicSensor);
    sensor_manager.add_sensor(MockGps);
    sensor_manager.add_sensor(MockVelocitySensor);
}

struct MockImuSensor;

impl HardwareSensor for MockImuSensor {
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

impl HardwareSensor for MockGps {
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

impl HardwareSensor for MockUltrasonicSensor {
    fn name(&self) -> SensorName {
        SensorName::Ultrasonic
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::Ultrasonic(rand::random::<f32>() * 10_f32)
    }
}

struct MockVelocitySensor;

impl HardwareSensor for MockVelocitySensor {
    fn name(&self) -> SensorName {
        SensorName::VirtualVelocity
    }

    fn read_data(&mut self) -> SensorData {
        SensorData::VirtualVelocity {
            imu: rand::random::<f64>() * 10.0,
            rot_encoder: rand::random::<f64>() * 10.0,
            fusion: rand::random::<f64>() * 10.0,
        }
    }
}
