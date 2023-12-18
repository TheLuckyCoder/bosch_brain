use std::net::UdpSocket;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tracing::warn;

use crate::sensors::{BasicSensor, GpsCoordinates, ImuData, SensorData, SensorManager};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UdpActiveSensor {
    Imu,
    Ultrasonic,
    Gps,
}

#[derive(Default)]
pub struct UdpBroadcast {
    active_sensors: Vec<UdpActiveSensor>,
    address: Option<String>,
    config_mode: bool,
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

impl UdpBroadcast {
    pub fn new(sensor_manager: Arc<Mutex<SensorManager>>) -> std::io::Result<Arc<Mutex<Self>>> {
        let udp_broadcast = Arc::new(Mutex::new(UdpBroadcast::default()));

        let server = UdpSocket::bind("0.0.0.0:3000")?;

        let udp_broadcast_clone = udp_broadcast.clone();

        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(100));

            let mut sensor_guard = sensor_manager.blocking_lock();
            let udp_guard = udp_broadcast.blocking_lock();

            let data = if udp_guard.config_mode {
                Self::reader_mode(&sensor_guard, &udp_guard)
            } else {
                Self::config_mode(&mut sensor_guard, &udp_guard)
            };
            
            let address = match &udp_guard.address {
                None => continue,
                Some(address) => address,
            }; 

            if let Some(data) = data {
                if let Err(err) = server.send_to(data.as_bytes(), address) {
                    warn!("Failed to send UDP Packet: {err}")
                }
            }
        });

        Ok(udp_broadcast_clone)
    }

    pub fn set_active_sensor(
        &mut self,
        sensor: Vec<UdpActiveSensor>,
        address: String,
    ) {
        self.active_sensors = sensor;
        self.address = Some(address);
    }
    
    pub fn set_config_mode(&mut self, config_mode: bool) {
        self.config_mode = config_mode;
    }

    fn reader_mode(sensor_manager: &SensorManager, udp_broadcast: &UdpBroadcast) -> Option<String> {
        let receiver = sensor_manager.get_data_receiver()?;

        let mut udp_data = receiver
            .try_iter()
            .fold(UdpData::default(), |mut udp, sensor_data| {
                match sensor_data.data {
                    SensorData::Imu(imu) => udp.imu = Some(imu),
                    SensorData::Distance(distance) => udp.ultrasonic = Some(distance),
                    SensorData::Gps(gps) => udp.gps = Some(gps),
                }
                udp
            });

        let active_sensors = &udp_broadcast.active_sensors;

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
            Some(serde_json::to_string(&udp_data).expect("Failed to serialize UDP data"))
        } else {
            None
        }
    }
    
    fn config_mode(sensor_manager: &mut SensorManager, udp_broadcast: &UdpBroadcast) -> Option<String> {
        let active_sensor = udp_broadcast.active_sensors.first()?;
        
        let sensor: &mut dyn BasicSensor = match active_sensor {
            UdpActiveSensor::Imu => sensor_manager.imu()?,
            UdpActiveSensor::Ultrasonic => sensor_manager.ultrasonic()?,
            UdpActiveSensor::Gps => sensor_manager.gps()?,
        };
        
        Some(sensor.read_config())
    }
}
