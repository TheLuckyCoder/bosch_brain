use std::net::UdpSocket;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tracing::warn;

use crate::sensors::{GpsCoordinates, ImuData, SensorData, SensorManager};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UdpActiveSensor {
    Imu,
    Ultrasonic,
    Gps,
}

#[derive(Default)]
pub struct UdpManager {
    active_sensors: Vec<UdpActiveSensor>,
    address: Option<String>,
}

#[derive(Default, serde::Serialize)]
struct UdpData {
    imu: Option<ImuData>,
    ultrasonic: Option<f32>,
    gps: Option<GpsCoordinates>,
}

impl UdpData {
    fn is_empty(&self) -> bool {
        self.imu.is_none() && self.ultrasonic.is_none() && self.gps.is_none()
    }
}

impl UdpManager {
    pub fn new(sensor_manager: Arc<Mutex<SensorManager>>) -> std::io::Result<Arc<Mutex<Self>>> {
        let udp_manager = Arc::new(Mutex::new(UdpManager::default()));

        let server = UdpSocket::bind("0.0.0.0:3000")?;

        let udp_manager_clone = udp_manager.clone();

        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(100));

            let guard = sensor_manager.blocking_lock();
            let receiver = match guard.get_data_receiver() {
                None => continue,
                Some(receiver) => receiver,
            };

            let mut udp_data =
                receiver
                    .try_iter()
                    .fold(UdpData::default(), |mut udp, sensor_data| {
                        match sensor_data.data {
                            SensorData::Imu(imu) => udp.imu = Some(imu),
                            SensorData::Distance(distance) => udp.ultrasonic = Some(distance),
                            SensorData::Gps(gps) => udp.gps = Some(gps),
                        }
                        udp
                    });

            let udp = udp_manager.blocking_lock();
            let address = match &udp.address {
                None => continue,
                Some(address) => address,
            };

            let active_sensors = &udp.active_sensors;

            if !active_sensors.contains(&UdpActiveSensor::Imu) {
                udp_data.imu = None;
            }

            if !active_sensors.contains(&UdpActiveSensor::Ultrasonic) {
                udp_data.ultrasonic = None;
            }

            if !active_sensors.contains(&UdpActiveSensor::Gps) {
                udp_data.gps = None;
            }

            if !udp_data.is_empty() {
                let json = serde_json::to_string(&udp_data).expect("Failed to serialize udp data");
                if let Err(err) = server.send_to(json.as_bytes(), address) {
                    warn!("Failed to send UDP Packet: {err}")
                }
            }
        });

        Ok(udp_manager_clone)
    }

    pub fn set_active_sensor(&mut self, sensor: Vec<UdpActiveSensor>, address: String) {
        self.active_sensors = sensor;
        self.address = Some(address);
    }
}
