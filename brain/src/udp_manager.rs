use serde_json::json;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sensors::SensorManager;

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum UdpActiveSensor {
    #[default]
    None,
    Imu,
    Distance,
}

#[derive(Default)]
pub struct UdpManager {
    active_sensor: Mutex<UdpActiveSensor>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UdpImuData {
    acceleration: [f32; 3],
    quaternion: [f32; 4],
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UdpData {
    imu: Option<UdpImuData>,
    distance: Option<f32>,
}

impl UdpManager {
    pub fn new(sensor_manager: Arc<SensorManager>) -> std::io::Result<Arc<Self>> {
        let udp_manager = Arc::new(Self::default());

        let server = UdpSocket::bind("0.0.0.0:3000")?;
        server.connect("0.0.0.0:3001")?;

        let udp_manager_clone = udp_manager.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(100));
            let active_sensor = *udp_manager.active_sensor.lock().unwrap();

            let data = match active_sensor {
                UdpActiveSensor::None => continue,
                UdpActiveSensor::Imu => {
                    let mut imu = sensor_manager.imu().lock().unwrap();

                    UdpData {
                        imu: Some(UdpImuData {
                            acceleration: imu.get_acceleration().into(),
                            quaternion: imu.get_quaternion().into(),
                        }),
                        distance: None,
                    }
                }
                UdpActiveSensor::Distance => {
                    let mut distance_sensor = sensor_manager.distance_sensor().lock().unwrap();

                    UdpData {
                        imu: None,
                        distance: Some(distance_sensor.get_distance_cm().unwrap_or(f32::NAN)),
                    }
                }
            };

            let json = serde_json::to_string(&data).expect("Failed to serialize udp data");
            server
                .send(json.as_bytes())
                .expect("Failed to send Udp Packet");
        });

        Ok(udp_manager_clone)
    }

    pub fn set_active_sensor(&self, sensor: UdpActiveSensor) {
        *self.active_sensor.lock().unwrap() = sensor;
    }
}

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
