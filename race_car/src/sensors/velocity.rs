use crate::sensors::{BasicSensor, ImuData, SensorData, SensorName, TimedSensorData};
use multiqueue2::BroadcastReceiver;
use shared::math::AlmostEquals;
use std::time::{SystemTime, UNIX_EPOCH};

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

    fn update_velocity(&mut self, data: ImuData, timestamp: SystemTime) {
        let acceleration = data.acceleration.x as f64;
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
                if let SensorData::Imu(imu_data) = sensor_data.data {
                    Some((imu_data, sensor_data.timestamp))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for (data, timestamp) in imu_data {
            self.update_velocity(data, timestamp);
        }

        SensorData::Velocity(self.last_velocity)
    }
}
