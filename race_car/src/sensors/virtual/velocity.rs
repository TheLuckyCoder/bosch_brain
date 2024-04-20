use std::time::{SystemTime, UNIX_EPOCH};

use multiqueue2::BroadcastReceiver;

use shared::math::AlmostEquals;

use crate::sensors::TimedSensorData;
use sensors::name::SensorName;
use sensors::{BasicSensor, SensorData};

pub struct VelocitySensor {
    receiver: BroadcastReceiver<TimedSensorData>,
    last_velocity: f64,
    last_acceleration: f64,
}

impl VelocitySensor {
    pub fn new(receiver: BroadcastReceiver<TimedSensorData>) -> Self {
        Self {
            receiver,
            last_velocity: 0.0,
            last_acceleration: 0.0,
        }
    }

    fn update_velocity(&mut self, acceleration: [f32; 3], timestamp: SystemTime) {
        let acceleration = acceleration[0] as f64;
        let acceleration = if acceleration.almost_equals(0.0, 0.006) {
            0.0
        } else {
            acceleration
        };
        // info!(
        //     "Acc: {acceleration}. LastVel: {}. LastAcc: {}",
        //     self.last_velocity, self.last_acceleration
        // );

        let seconds = timestamp
            .duration_since(UNIX_EPOCH)
            .expect("Invalid Timestamp")
            .as_secs_f64();
        let velocity =
            self.last_velocity + 0.5f64 * (acceleration + self.last_acceleration) * seconds;

        self.last_velocity = velocity;
        self.last_acceleration = acceleration;
    }
}

impl BasicSensor for VelocitySensor {
    fn name(&self) -> SensorName {
        SensorName::Velocity
    }

    fn prepare_read(&mut self) {
        self.last_velocity = 0.0;
        self.last_acceleration = 0.0;
    }

    fn read_data(&mut self) -> SensorData {
        let imu_data = self
            .receiver
            .try_iter()
            .filter_map(|sensor_data: TimedSensorData| {
                if let SensorData::Imu { acceleration, .. } = sensor_data.data {
                    Some((acceleration, sensor_data.timestamp))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for (acceleration, timestamp) in imu_data {
            self.update_velocity(acceleration, timestamp);
        }

        SensorData::Velocity(self.last_velocity)
    }
}
