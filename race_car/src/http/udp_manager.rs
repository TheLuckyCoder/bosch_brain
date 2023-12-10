use std::net::UdpSocket;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::sensors::SensorManager;
use tracing::warn;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UdpActiveSensor {
    Imu,
    Ultrasonic,
}

#[derive(Default)]
pub struct UdpManager {
    active_sensors: Mutex<Vec<UdpActiveSensor>>,
    address: Mutex<Option<String>>,
}

#[derive(serde::Serialize)]
struct UdpImuData {
    acceleration: [f32; 3],
    quaternion: [f32; 4],
}

#[derive(serde::Serialize)]
struct UdpData {
    imu: Option<UdpImuData>,
    distance: Option<f32>,
}

impl UdpManager {
    pub fn new(sensor_manager: Arc<Mutex<SensorManager>>) -> std::io::Result<Arc<Self>> {
        let udp_manager = Arc::new(Self::default());

        let server = UdpSocket::bind("0.0.0.0:3000")?;

        let udp_manager_clone = udp_manager.clone();

        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(100));

            let address = match udp_manager.address.blocking_lock().clone() {
                None => continue,
                Some(address) => address,
            };

            let active_sensors = udp_manager.active_sensors.blocking_lock().clone();

            let mut udp_data = UdpData {
                imu: None,
                distance: None,
            };

            let mut guard = sensor_manager.blocking_lock();
            for sensor in active_sensors {
                match sensor {
                    UdpActiveSensor::Imu => {
                        // let imu = guard.imu().unwrap();
                        //
                        // udp_data.imu = Some(UdpImuData {
                        //     acceleration: imu.get_acceleration().into(),
                        //     quaternion: imu.get_quaternion().into(),
                        // });
                    }
                    UdpActiveSensor::Ultrasonic => {
                        // let distance_sensor = guard.distance_sensor().unwrap();

                        // udp_data.distance =
                        //     Some(distance_sensor.get_distance_cm().unwrap_or(f32::NAN));
                    }
                };
            }

            if udp_data.imu.is_some() || udp_data.distance.is_some() {
                let json = serde_json::to_string(&udp_data).expect("Failed to serialize udp data");
                if let Err(err) = server.send_to(json.as_bytes(), address) {
                    warn!("Failed to send UDP Packet: {err}")
                }
            }
        });

        Ok(udp_manager_clone)
    }

    pub fn set_active_sensor(&self, sensor: Vec<UdpActiveSensor>, address: String) {
        *self.active_sensors.blocking_lock() = sensor;
        *self.address.blocking_lock() = Some(address);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_deserialize() {
        let json = json!(
            {
                "imu": {
                    "acceleration": [1, 2, 3],
                    "quaternion": [1, 2, 3, 4],
                },
                "distance": 23.5
            }
        );

        let data: UdpData = serde_json::from_value(json).unwrap();
    }
}
