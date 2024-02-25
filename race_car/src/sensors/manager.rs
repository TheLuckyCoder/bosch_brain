use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::TrySendError;
use std::thread;
use std::time::{Duration, SystemTime};

use multiqueue2::{broadcast_queue, BroadcastReceiver, BroadcastSender};
use tracing::{error, warn};

use crate::sensors::{AmbienceSensor, BasicSensor, GpsSensor, ImuSensor, SensorData, SensorName, set_board_led_status, TimedSensorData, UltrasonicSensor};
use crate::sensors::velocity::VelocitySensor;

struct Shared {
    should_read: AtomicBool,
    start_time: Mutex<SystemTime>,
    cond: Condvar,
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            should_read: Default::default(),
            start_time: Mutex::new(SystemTime::now()),
            cond: Default::default(),
        }
    }
}

/// Manages all the sensor instances
pub struct SensorManager {
    shared_data: Arc<Shared>,
    sensors: HashMap<SensorName, Arc<Mutex<dyn BasicSensor + Send>>>,
    receiver: BroadcastReceiver<TimedSensorData>,
}

impl SensorManager {
    pub fn new() -> Self {
        let shared_data = Arc::new(Shared::default());
        let mut sensors = HashMap::new();
        let (sender, receiver) = broadcast_queue(32);

        let mut spawn_thread = |sensor: Arc<Mutex<dyn BasicSensor + Send>>| {
            let sensor_name = sensor.lock().unwrap().name();
            Self::spawn_sensor_thread(
                sensor_name,
                sensor.clone(),
                shared_data.clone(),
                sender.clone(),
            );

            sensors.insert(sensor_name, sensor);
        };

        fn cast_sensor(sensor: impl BasicSensor + 'static) -> Arc<Mutex<dyn BasicSensor + Send>> {
            Arc::new(Mutex::new(sensor)) as Arc<Mutex<dyn BasicSensor + Send>>
        }

        // Initialize the actual sensors
        ImuSensor::new()
            .map(cast_sensor)
            .map(&mut spawn_thread)
            .map_err(|e| error!("IMU failed to initialize: {e:?}"))
            .ok();
        spawn_thread(cast_sensor(VelocitySensor::new(receiver.clone())));
        // UltrasonicSensor::new(21f32)
        //     .map(cast_sensor)
        //     .map(&mut spawn_thread)
        //     .map_err(|e| error!("Ultrasonic Sensor failed to initialize: {e:?}"))
        //     .ok();
        GpsSensor::new()
            .map(cast_sensor)
            .map(&mut spawn_thread)
            .map_err(|e| error!("GPS failed to initialize: {e}"))
            .ok();
        // AmbienceSensor::new()
        //     .map(cast_sensor)
        //     .map(&mut spawn_thread)
        //     .map_err(|e| error!("AmbienceSensor failed to initialize: {e:?}"))
        //     .ok();

        Self { shared_data, sensors, receiver }
    }

    pub fn get_sensor(&self, sensor_name: &SensorName) -> Option<&Mutex<dyn BasicSensor + Send>> {
        self.sensors.get(sensor_name).map(|sensor| sensor.as_ref())
    }

    fn spawn_sensor_thread(
        sensor_name: SensorName,
        sensor: Arc<Mutex<dyn BasicSensor + Send>>,
        shared_data: Arc<Shared>,
        sender: BroadcastSender<TimedSensorData>,
    ) {
        thread::spawn(move || {
            let mut start_time = SystemTime::now();

            loop {
                if !shared_data.should_read.load(Ordering::Acquire) {
                    let mut lock = shared_data.start_time.lock().unwrap();
                    while !shared_data.should_read.load(Ordering::Acquire) {
                        lock = shared_data.cond.wait(lock).unwrap();
                    }

                    // Now is the start of a new reading session
                    start_time = *lock;
                    sensor.lock().unwrap().prepare_read();
                }

                if sensor_name != SensorName::Gps {
                    thread::sleep(Duration::from_millis(20));
                }

                let sensor_data = sensor.lock().unwrap().read_data_timed(start_time);

                // if sensor_name == Gps::NAME {
                //     info!(
                //         "{:?} {} elapsed {}ms",
                //         sensor_data.data,
                //         sensor.name(),
                //         time_elapsed.as_millis(),
                //     );
                // }

                if !shared_data.should_read.load(Ordering::Acquire) {
                    continue;
                }

                if let Err(e) = sender.try_send(sensor_data) {
                    match e {
                        TrySendError::Full(_) => {
                            warn!("{sensor_name} channel is full, failed to send new sensor data");
                            continue;
                        }
                        TrySendError::Disconnected(_) => {
                            error!("{sensor_name} channel disconnected");
                            break;
                        }
                    }
                }
            }
        });
    }

    pub fn start_listening_to_sensors(&mut self) {
        self.shared_data.should_read.store(true, Ordering::Release);
        self.shared_data.cond.notify_all();
    }

    pub fn stop_listening_to_sensors(&mut self) {
        self.shared_data.should_read.store(false, Ordering::Release)
    }

    pub fn get_data_receiver(&self) -> &BroadcastReceiver<TimedSensorData> {
        &self.receiver
    }
}
