use std::time::Instant;
use multiqueue2::BroadcastReceiver;
use crate::sensors::{BasicSensor, ImuData, SensorData, SensorName, TimedSensorData};

pub struct VelocitySensor {
    receiver: BroadcastReceiver<TimedSensorData>,
    last_velocity: f64,
    last_acceleration: f64,
    since_last_read: Instant,
}

impl VelocitySensor {
    pub fn new(receiver: BroadcastReceiver<TimedSensorData>) -> Self {
        Self {
            receiver,
            last_velocity: 0.0,
            last_acceleration: 0.0,
            since_last_read: Instant::now(),
        }
    }

    fn update_velocity(&mut self, data: ImuData) {
        let acceleration = data.acceleration.x as f64;
        let acceleration = if acceleration < 0.005 {
            0.0
        } else {
            acceleration
        };

        let velocity = self.last_velocity
            + 0.5f64
            * (acceleration + self.last_acceleration)
            * self.since_last_read.elapsed().as_secs_f64();

        self.last_velocity = velocity;
        self.last_acceleration = acceleration;
        // TODO Maybe use the Time from the ImuData
        self.since_last_read = Instant::now();
    }
}

impl BasicSensor for VelocitySensor {
    fn name(&self) -> SensorName {
        SensorName::Velocity
    }

    fn prepare_read(&mut self) {
        self.last_velocity = 0.0;
        self.last_acceleration = 0.0;
        self.since_last_read = Instant::now();
    }

    fn read_data(&mut self) -> SensorData {
        let imu_data = self.receiver.try_iter()
            .filter_map(|sensor_data: TimedSensorData| {
                if let SensorData::Imu(imu_data) = sensor_data.data {
                    Some(imu_data)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for data in imu_data {
            self.update_velocity(data);
        }

        SensorData::Velocity(self.last_velocity)
    }
}
