use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::TrySendError;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Instant, SystemTime};

use multiqueue2::{broadcast_queue, BroadcastReceiver, BroadcastSender};
use tracing::{error, info, warn};

use crate::sensors::ambience::AmbienceSensor;
use crate::sensors::{
    set_board_led_status, BasicSensor, Gps, Imu, SensorData, TimedSensorData, UltrasonicSensor,
};

enum ManagerState {
    Normal {
        imu: Option<Box<Imu>>,
        ultrasonic: Option<Box<UltrasonicSensor>>,
        gps: Option<Box<Gps>>,
        ambience: Option<Box<AmbienceSensor>>,
    },
    Reading {
        is_active: Arc<AtomicBool>,
        handles: Vec<JoinHandle<()>>,
        receiver: BroadcastReceiver<TimedSensorData>,
    },
}

/// Manages all the sensor instances
pub struct SensorManager {
    state: ManagerState,
}

impl SensorManager {
    pub fn new() -> Self {
        let imu = Imu::new()
            .map(Box::new)
            .map_err(|e| error!("IMU failed to initialize: {e:?}"))
            .ok();
        // let ultrasonic = UltrasonicSensor::new(21f32)
        //     .map(Box::new)
        //     .map_err(|e| error!("Ultrasonic Sensor failed to initialize: {e:?}"))
        //     .ok();
        let gps = Gps::new()
            .map(Box::new)
            .map_err(|e| error!("GPS failed to initialize: {e}"))
            .ok();
        // let ambience = AmbienceSensor::new()
        //     .map(Box::new)
        //     .map_err(|e| error!("Ambience failed to initialize: {e:?}"))
        //     .ok();

        let state = ManagerState::Normal {
            imu,
            ultrasonic: None,
            gps,
            ambience: None,
        };

        Self { state }
    }

    /// Gives a mutable reference to the IMU sensor if in the default state
    pub fn imu(&mut self) -> Option<&mut Imu> {
        match &mut self.state {
            ManagerState::Normal { imu, .. } => imu.as_deref_mut(),
            ManagerState::Reading { .. } => None,
        }
    }

    /// Gives a mutable reference to the Ultrasonic sensor if in the default state
    pub fn ultrasonic(&mut self) -> Option<&mut UltrasonicSensor> {
        match &mut self.state {
            ManagerState::Normal { ultrasonic, .. } => ultrasonic.as_deref_mut(),
            ManagerState::Reading { .. } => None,
        }
    }

    /// Gives a mutable reference to the GPS sensor if in the default state
    pub fn gps(&mut self) -> Option<&mut Gps> {
        match &mut self.state {
            ManagerState::Normal { gps, .. } => gps.as_deref_mut(),
            ManagerState::Reading { .. } => None,
        }
    }

    /// Gives a mutable reference to the Ambience sensor if in the default state
    pub(crate) fn ambience(&mut self) -> Option<&mut AmbienceSensor> {
        if let ManagerState::Normal { ambience, .. } = &mut self.state {
            ambience.as_deref_mut()
        } else {
            None
        }
    }

    fn spawn_sensor_thread(
        mut sensor: Box<dyn BasicSensor + Send>,
        is_active: Arc<AtomicBool>,
        sender: BroadcastSender<TimedSensorData>,
        start_time: SystemTime,
    ) -> JoinHandle<()> {
        thread::Builder::new()
            .name(sensor.name().to_string())
            .spawn(move || {
                let mut since_last_read = Instant::now();
                let mut previous_velocity = 0f64;
                let mut previous_acceleration = 0f64;
                let mut updated_count = 0usize;

                while is_active.load(Ordering::Acquire) {
                    let instant = Instant::now();
                    let sensor_data = sensor.read_data_timed(start_time);
                    let time_elapsed = instant.elapsed();

                    if let SensorData::Imu(data) = &sensor_data.data {
                        let acceleration = data.acceleration.x as f64;
                        let acceleration = if acceleration < 0.005 {
                            0.0
                        } else {
                            acceleration
                        };

                        let velocity = previous_velocity
                            + 0.5f64
                                * (acceleration + previous_acceleration)
                                * since_last_read.elapsed().as_secs_f64();

                        if updated_count % 10 == 0 {
                            info!("Velocity: {velocity}");
                        }

                        since_last_read = Instant::now();
                        previous_acceleration = acceleration;
                        previous_velocity = velocity;
                        updated_count += 1;

                        if let Err(e) =
                            sender.try_send(TimedSensorData::from(SensorData::Velocity(velocity)))
                        {
                            error!("{e}")
                        }
                    }

                    if sensor.name() == Gps::NAME {
                        info!(
                            "{:?} {} elapsed {}ms",
                            sensor_data.data,
                            sensor.name(),
                            time_elapsed.as_millis(),
                        );
                    }

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

                    if sensor.name() != Gps::NAME {
                        thread::sleep(std::time::Duration::from_millis(20));
                    }
                }
            })
            .unwrap()
    }

    pub fn listen_to_all_sensors(&mut self) -> BroadcastReceiver<TimedSensorData> {
        match &mut self.state {
            ManagerState::Normal {
                imu,
                ultrasonic,
                gps,
                ambience,
            } => {
                let (sender, receiver) = broadcast_queue(32);

                let is_active = Arc::new(AtomicBool::new(true));
                let mut handles = vec![];

                set_board_led_status(true).unwrap();
                let start_time = SystemTime::now();
                let mut spawn_thread = |sensor: Box<dyn BasicSensor + Send>| {
                    handles.push(Self::spawn_sensor_thread(
                        sensor,
                        is_active.clone(),
                        sender.clone(),
                        start_time,
                    ));
                };

                if let Some(sensor) = imu.take() {
                    spawn_thread(sensor)
                }
                if let Some(sensor) = ultrasonic.take() {
                    spawn_thread(sensor)
                }
                if let Some(sensor) = gps.take() {
                    spawn_thread(sensor)
                }
                if let Some(sensor) = ambience.take() {
                    spawn_thread(sensor)
                }

                self.state = ManagerState::Reading {
                    is_active,
                    handles,
                    receiver: receiver.add_stream(),
                };
                receiver
            }
            ManagerState::Reading { .. } => panic!("Wrong state!"),
        }
    }

    pub fn get_data_receiver(&self) -> Option<&BroadcastReceiver<TimedSensorData>> {
        match &self.state {
            ManagerState::Normal { .. } => None,
            ManagerState::Reading { receiver, .. } => Some(receiver),
        }
    }

    /// Resets the internal state of the sensor manager
    pub fn reset(&mut self) {
        info!("Resetting");

        match &mut self.state {
            ManagerState::Normal {
                imu,
                ultrasonic,
                gps,
                ambience,
            } => {
                imu.take();
                ultrasonic.take();
                gps.take();
                ambience.take();
            }
            ManagerState::Reading {
                is_active, handles, ..
            } => {
                is_active.store(false, Ordering::Release);

                let handles = std::mem::take(handles);
                handles.into_iter().for_each(|handle| {
                    handle.join().unwrap();
                });
                info!("Finished closing sensor threads");
            }
        }

        self.state = SensorManager::new().state;
        info!("Recreated SensorManager");

        set_board_led_status(false).unwrap();
    }
}
