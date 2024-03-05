//! Handles the UDP broadcast of the sensor data

use std::net::UdpSocket;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::sensors::manager::SensorManager;
use crate::sensors::{AmbienceData, GpsCoordinates, ImuData, SensorData, SensorName};

/// The data that is sent over UDP
#[derive(Default, serde::Serialize)]
struct UdpData {
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    imu: Option<ImuData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ultrasonic: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gps: Option<GpsCoordinates>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ambience: Option<AmbienceData>,
}

impl UdpData {
    /// Checks if the struct contains any data
    fn is_empty(&self) -> bool {
        self.imu.is_none()
            && self.ultrasonic.is_none()
            && self.gps.is_none()
            && self.ambience.is_none()
    }
}

/// Handles the UDP broadcast of the sensor data
#[derive(Default)]
pub struct UdpBroadcast {
    active_sensors: Vec<SensorName>,
    address: Option<String>,
    config_mode: bool,
}

impl UdpBroadcast {
    /// Creates a new UDP broadcaster, there should only exist one instance of this.
    ///
    /// This starts a background thread which periodically checks for new sensor data and sends it over UDP on port `3000`.
    ///
    /// This will only send data after the `address` has been set. See [UdpBroadcast::set_active_sensor] for more information.
    pub fn new(sensor_manager: Arc<Mutex<SensorManager>>) -> std::io::Result<Arc<Mutex<Self>>> {
        let udp_broadcast = Arc::new(Mutex::new(UdpBroadcast::default()));

        let server = UdpSocket::bind("0.0.0.0:3000")?;

        let udp_broadcast_clone = udp_broadcast.clone();

        std::thread::Builder::new()
            .name(String::from("UDP Broadcaster"))
            .spawn(move || loop {
                std::thread::sleep(Duration::from_millis(50));

                let mut sensor_guard = sensor_manager.blocking_lock();
                let udp_guard = udp_broadcast.blocking_lock();

                let data = if udp_guard.config_mode {
                    udp_guard.config_mode(&mut sensor_guard)
                } else {
                    udp_guard.reader_mode(&sensor_guard)
                };
                drop(sensor_guard);

                let address = match &udp_guard.address {
                    None => continue,
                    Some(address) => address,
                };

                if let Some(data) = data {
                    // info!("Udp: {data}");
                    if let Err(err) = server.send_to(data.as_bytes(), address) {
                        warn!("Failed to send UDP Packet: {err}")
                    }
                }
            })
            .unwrap();

        Ok(udp_broadcast_clone)
    }

    pub fn set_active_sensor(&mut self, sensor: Vec<SensorName>, address: String) {
        self.active_sensors = sensor;
        self.address = Some(address);
    }

    pub fn set_config_mode(&mut self, config_mode: bool) {
        self.config_mode = config_mode;
    }

    pub fn save_sensor_config(&self, sensor_manager: &mut SensorManager) -> Option<()> {
        if !self.config_mode {
            return None;
        }

        let active_sensor = self.active_sensors.first()?;

        let sensor = sensor_manager.get_sensor(active_sensor)?;

        if let Err(e) = sensor.lock().unwrap().save_config() {
            error!("Failed to save error: {e}");
        }

        Some(())
    }

    fn reader_mode(&self, sensor_manager: &SensorManager) -> Option<String> {
        let receiver = sensor_manager.get_data_receiver();

        let mut udp_data = receiver
            .try_iter()
            .fold(UdpData::default(), |mut udp, sensor_data| {
                match sensor_data.data {
                    SensorData::Imu(imu) => udp.imu = Some(imu),
                    SensorData::Distance(distance) => udp.ultrasonic = Some(distance),
                    SensorData::Gps(gps) => udp.gps = Some(gps),
                    SensorData::Ambience(ambience) => udp.ambience = Some(ambience),
                    _ => {}
                }
                udp
            });

        let active_sensors = &self.active_sensors;

        if !active_sensors.contains(&SensorName::Imu) {
            udp_data.imu = None;
        }

        if !active_sensors.contains(&SensorName::Ultrasonic) {
            udp_data.ultrasonic = None;
        }

        if !active_sensors.contains(&SensorName::Gps) {
            udp_data.gps = None;
        }

        if !active_sensors.contains(&SensorName::Ambience) {
            udp_data.ambience = None;
        }

        if !udp_data.is_empty() {
            Some(serde_json::to_string(&udp_data).expect("Failed to serialize UDP data"))
        } else {
            None
        }
    }

    fn config_mode(&self, sensor_manager: &mut SensorManager) -> Option<String> {
        let active_sensor = self.active_sensors.first()?;

        let sensor = sensor_manager.get_sensor(active_sensor)?;

        Some(sensor.lock().unwrap().read_debug())
    }
}
