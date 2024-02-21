use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::TrySendError;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime};

use multiqueue2::{broadcast_queue, BroadcastReceiver, BroadcastSender};
use tracing::{error, info, warn};

use crate::sensors::{set_board_led_status, BasicSensor, Gps, Imu, SensorData, TimedSensorData, SensorName, UltrasonicSensor, AmbienceSensor};

struct Shared {
    actively_reading: AtomicBool,
}

struct SensorContainer {
    sensor: Arc<Mutex<dyn BasicSensor + Send>>,
    handle: JoinHandle<()>,
}

/// Manages all the sensor instances
pub struct SensorManager {
    should_read: Arc<AtomicBool>,
    sensors: HashMap<SensorName, SensorContainer>,
    receiver: BroadcastReceiver<TimedSensorData>,
}

impl SensorManager {
    pub fn new() -> Self {
        let should_read = Arc::new(AtomicBool::new(true));
        let (sender, receiver) = broadcast_queue(32);

        let mut manager = Self { should_read: should_read.clone(), sensors: HashMap::new(), receiver };

        set_board_led_status(true).unwrap();
        let start_time = SystemTime::now();

        let mut spawn_thread = |sensor: Arc<Mutex<dyn BasicSensor + Send>>| {
            let handle = Self::spawn_sensor_thread(
                sensor.clone(),
                should_read.clone(),
                sender.clone(),
                start_time,
            );

            let sensor_name = sensor.lock().unwrap().name();
            manager.sensors.insert(sensor_name, SensorContainer { sensor, handle });
        };

        fn cast_sensor(sensor: impl BasicSensor + Send + 'static) -> Arc<Mutex<dyn BasicSensor + Send>> {
            Arc::new(Mutex::new(sensor)) as Arc<Mutex<dyn BasicSensor + Send>>
        }

        // Initialize the actual sensors
        Imu::new()
            .map(cast_sensor)
            .map(&mut spawn_thread)
            .map_err(|e| error!("IMU failed to initialize: {e:?}"))
            .ok();
        // UltrasonicSensor::new(21f32)
        //     .map(cast_sensor)
        //     .map(&mut spawn_thread)
        //     .map_err(|e| error!("Ultrasonic Sensor failed to initialize: {e:?}"))
        //     .ok();
        Gps::new()
            .map(cast_sensor)
            .map(&mut spawn_thread)
            .map_err(|e| error!("GPS failed to initialize: {e}"))
            .ok();
        // AmbienceSensor::new()
        //     .map(cast_sensor)
        //     .map(&mut spawn_thread)
        //     .map_err(|e| error!("AmbienceSensor failed to initialize: {e:?}"))
        //     .ok();

        manager
    }

    pub fn get_sensor(&self, sensor_name: &SensorName) -> Option<&Mutex<dyn BasicSensor + Send>> {
        self.sensors.get(sensor_name).map(|container| container.sensor.as_ref())
    }

    fn spawn_sensor_thread(
        sensor: Arc<Mutex<dyn BasicSensor + Send>>,
        is_active: Arc<AtomicBool>,
        sender: BroadcastSender<TimedSensorData>,
        start_time: SystemTime,
    ) -> JoinHandle<()> {
        let sensor_name = sensor.lock().unwrap().name();
        thread::Builder::new()
            .name(sensor_name.as_ref().to_string())
            .spawn(move || {
                let mut since_last_read = Instant::now();
                let mut previous_velocity = 0f64;
                let mut previous_acceleration = 0f64;
                let mut updated_count = 0usize;

                loop {
                    if !is_active.load(Ordering::Acquire) {
                        thread::sleep(Duration::from_millis(10))
                    }

                    if sensor_name != SensorName::Gps {
                        thread::sleep(Duration::from_millis(20));
                    }

                    let sensor_data = sensor.lock().unwrap().read_data_timed(start_time);

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
                            sender.try_send(TimedSensorData::new(SensorData::Velocity(velocity), start_time))
                        {
                            error!("{e}")
                        }
                    }

                    // if sensor.name() == Gps::NAME {
                    //     info!(
                    //         "{:?} {} elapsed {}ms",
                    //         sensor_data.data,
                    //         sensor.name(),
                    //         time_elapsed.as_millis(),
                    //     );
                    // }

                    if !is_active.load(Ordering::Acquire) {
                        continue;
                    }

                    if let Err(e) = sender.try_send(sensor_data) {
                        match e {
                            TrySendError::Full(_) => {
                                warn!(
                                    "{sensor_name} channel is full, failed to send new sensor data",
                                );
                                continue;
                            }
                            TrySendError::Disconnected(_) => {
                                error!("{sensor_name} channel disconnected");
                                break;
                            }
                        }
                    }
                }
            })
            .unwrap()
    }

    pub fn listen_to_all_sensors(&mut self) {
        self.should_read.store(true, Ordering::Release);
    }

    pub fn get_data_receiver(&self) -> &BroadcastReceiver<TimedSensorData> {
        &self.receiver
    }

    /// Resets the internal state of the sensor manager
    pub fn reset(&mut self) {
        info!("Resetting");

        info!("Recreated SensorManager");

        set_board_led_status(false).unwrap();
    }
}
