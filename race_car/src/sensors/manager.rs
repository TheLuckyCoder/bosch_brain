use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::TrySendError;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use multiqueue2::{broadcast_queue, BroadcastReceiver, BroadcastSender};
use tracing::{error, info, warn};

use crate::sensors::gps::Gps;
use crate::sensors::{BasicSensor, ImuSensor, TimedSensorData, UltrasonicSensor};

enum ManagerState {
    Normal {
        imu: Option<ImuSensor>,
        ultrasonic: Option<UltrasonicSensor>,
        gps: Option<Gps>,
    },
    Reading {
        is_active: Arc<AtomicBool>,
        handles: Vec<JoinHandle<()>>,
    },
}

pub struct SensorManager {
    state: ManagerState,
}

impl SensorManager {
    pub fn new() -> Self {
        let imu = ImuSensor::new()
            .map_err(|e| error!("IMU failed to initialize: {e}"))
            .ok();
        let ultrasonic = UltrasonicSensor::new(21f32)
            .map_err(|e| error!("Ultrasonic Sensor failed to initialize: {e}"))
            .ok();
        let gps = Gps::new()
            .map_err(|e| error!("GPS failed to initialize: {e}"))
            .ok();

        let state = ManagerState::Normal {
            imu,
            ultrasonic,
            gps,
        };

        Self { state }
    }

    pub fn imu(&self) -> Option<&ImuSensor> {
        match &self.state {
            ManagerState::Normal { imu, .. } => imu.as_ref(),
            ManagerState::Reading { .. } => None,
        }
    }

    pub fn distance_sensor(&self) -> Option<&UltrasonicSensor> {
        match &self.state {
            ManagerState::Normal { ultrasonic, .. } => ultrasonic.as_ref(),
            ManagerState::Reading { .. } => None,
        }
    }

    pub fn get_active_sensors(&self) -> Vec<&dyn BasicSensor> {
        match &self.state {
            ManagerState::Normal {
                imu,
                ultrasonic,
                gps,
            } => {
                let sensors = [
                    imu.as_ref().map(|x| x as &dyn BasicSensor),
                    ultrasonic.as_ref().map(|x| x as &dyn BasicSensor),
                    gps.as_ref().map(|x| x as &dyn BasicSensor),
                ];

                sensors.into_iter().flatten().collect()
            }
            ManagerState::Reading { .. } => Vec::new(),
        }
    }

    fn spawn_reading_thread(
        mut sensor: impl BasicSensor + Send + 'static,
        is_active: Arc<AtomicBool>,
        sender: BroadcastSender<TimedSensorData>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            while is_active.load(Ordering::Acquire) {
                let instant = Instant::now();
                let sensor_data = sensor.read_data_timed();
                let time_elapsed = instant.elapsed();

                info!(
                    "{} elapsed time({}ms) data({sensor_data:?})",
                    sensor.name(),
                    time_elapsed.as_millis()
                );

                if !is_active.load(Ordering::Acquire) {
                    break;
                }

                if let Err(e) = sender.try_send(sensor_data) {
                    match e {
                        TrySendError::Full(_) => {
                            warn!(
                                "{} channel is full, failed to send new sensor data",
                                sensor.name()
                            );
                            continue;
                        }
                        TrySendError::Disconnected(_) => {
                            error!("{} channel disconnected", sensor.name());
                            break;
                        }
                    }
                }

                thread::sleep(Duration::from_millis(200)); // TODO Remove
            }
        })
    }

    pub fn listen_to_all_sensors(&mut self) -> BroadcastReceiver<TimedSensorData> {
        match &mut self.state {
            ManagerState::Normal {
                imu,
                ultrasonic,
                gps,
            } => {
                let (tx, rx) = broadcast_queue(16);

                let is_active = Arc::new(AtomicBool::new(true));
                let handles = vec![];

                if let Some(imu) = imu.take() {
                    Self::spawn_reading_thread(imu, is_active.clone(), tx.clone());
                }
                if let Some(ultrasonic) = ultrasonic.take() {
                    Self::spawn_reading_thread(ultrasonic, is_active.clone(), tx.clone());
                }
                if let Some(gps) = gps.take() {
                    Self::spawn_reading_thread(gps, is_active.clone(), tx);
                }

                self.state = ManagerState::Reading { is_active, handles };

                rx
            }
            ManagerState::Reading { .. } => panic!("Wrong state!"),
        }
    }

    pub fn reset(&mut self) {
        match &mut self.state {
            ManagerState::Normal {
                imu,
                ultrasonic,
                gps,
            } => {
                drop(imu.take());
                drop(ultrasonic.take());
                drop(gps.take());
            }
            ManagerState::Reading { is_active, handles } => {
                is_active.store(false, Ordering::Release);

                let mut h = vec![];
                std::mem::swap(handles, &mut h);
                h.into_iter().for_each(|handle| {
                    handle.join().unwrap();
                });
            }
        }

        self.state = SensorManager::new().state;
    }
}
