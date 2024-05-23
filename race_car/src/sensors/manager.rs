use std::{mem, thread};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::TrySendError;
use std::thread::JoinHandle;
use std::time::Duration;

use multiqueue2::{broadcast_queue, BroadcastReceiver, BroadcastSender};
use tracing::{debug, error, info, warn};

use sensors::BasicSensor;
use sensors::name::SensorName;

use crate::sensors::TimedSensorData;

#[derive(Default)]
struct Shared {
    should_read: AtomicBool,
    is_stopped: AtomicBool,
    lock: Mutex<()>,
    cond: Condvar,
}

/// Manages all the sensor instances
pub struct SensorManager {
    shared_data: Arc<Shared>,
    sensors: BTreeMap<SensorName, Arc<Mutex<dyn BasicSensor + Send>>>,
    handles: HashMap<SensorName, JoinHandle<()>>,
    receiver: BroadcastReceiver<TimedSensorData>,
    sender: BroadcastSender<TimedSensorData>,
}

impl SensorManager {
    pub fn new() -> Self {
        let (sender, receiver) = broadcast_queue(64);
        Self {
            shared_data: Arc::new(Shared::default()),
            sensors: BTreeMap::new(),
            handles: HashMap::new(),
            receiver,
            sender,
        }
    }

    pub fn add_sensor(&mut self, sensor: impl BasicSensor) {
        let sensor_name = sensor.name();
        let sensor = Arc::new(Mutex::new(sensor)) as Arc<Mutex<dyn BasicSensor + Send>>;

        let handle = Self::spawn_sensor_thread(
            sensor_name,
            sensor.clone(),
            self.shared_data.clone(),
            self.sender.clone(),
        );

        self.sensors.insert(sensor_name, sensor);
        self.handles.insert(sensor_name, handle);
    }

    pub fn get_sensor(&self, sensor_name: &SensorName) -> Option<&Mutex<dyn BasicSensor + Send>> {
        self.sensors.get(sensor_name).map(|sensor| sensor.as_ref())
    }

    pub fn remove_all_sensors(&mut self) {
        self.shared_data.is_stopped.store(true, Ordering::Release);

        // Wake the threads in order to notice the is_stopped variable
        self.start_listening_to_sensors();

        mem::take(&mut self.handles)
            .into_iter()
            .for_each(|(name, handle)| {
                debug!("Waiting for {name} to join");
                handle.join().expect("Error occurred while joining sensor");
            });

        info!("Finished joining all existing sensors");

        self.shared_data = Arc::new(Shared::default());
        self.sensors.clear();
        self.receiver.try_iter().for_each(drop);
    }

    pub fn start_listening_to_sensors(&self) {
        let _lock = self.shared_data.lock.lock();
        self.shared_data.should_read.store(true, Ordering::Release);
        self.shared_data.cond.notify_all();
    }

    pub fn stop_listening_to_sensors(&self) {
        self.shared_data.should_read.store(false, Ordering::Release)
    }

    pub fn get_data_receiver(&self) -> &BroadcastReceiver<TimedSensorData> {
        &self.receiver
    }

    pub fn get_active_sensors(&self) -> Vec<(SensorName, &Mutex<dyn BasicSensor + Send>)> {
        self.sensors
            .iter()
            .map(|(name, sensor)| (*name, sensor.as_ref()))
            .collect::<Vec<_>>()
    }

    fn spawn_sensor_thread(
        sensor_name: SensorName,
        sensor: Arc<Mutex<dyn BasicSensor + Send>>,
        shared_data: Arc<Shared>,
        sender: BroadcastSender<TimedSensorData>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            loop {
                if shared_data.is_stopped.load(Ordering::Acquire) {
                    return;
                }

                if !shared_data.should_read.load(Ordering::Acquire) {
                    let mut lock = shared_data.lock.lock().unwrap();
                    while !shared_data.should_read.load(Ordering::Acquire) {
                        lock = shared_data.cond.wait(lock).unwrap();
                    }

                    // Now is the start of a new reading session
                    sensor.lock().unwrap().prepare_read();

                    if shared_data.is_stopped.load(Ordering::Acquire) {
                        return;
                    }
                }

                thread::sleep(Duration::from_millis(50));

                let sensor_data = TimedSensorData::from(sensor.lock().unwrap().read_data());

                // println!("{:?}: {}", sensor_data.data, sensor_name);

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
        })
    }
}

impl Drop for SensorManager {
    fn drop(&mut self) {
        self.stop_listening_to_sensors();
    }
}

#[cfg(test)]
mod tests {
    use sensors::SensorData::Velocity;

    use crate::sensors::SensorData;

    use super::*;

    struct TestSensor {
        name: SensorName,
        data: Vec<SensorData>,
        index: usize,
    }

    impl TestSensor {
        fn new(name: SensorName, data: Vec<SensorData>) -> Self {
            Self {
                name,
                data,
                index: 0,
            }
        }
    }

    impl BasicSensor for TestSensor {
        fn name(&self) -> SensorName {
            self.name
        }

        fn read_data(&mut self) -> SensorData {
            let data = self.data[self.index % self.data.len()].clone();
            self.index += 1;
            data
        }
    }

    #[test]
    fn test_sensor_manager() {
        let data_list = vec![
            Velocity(10.0),
            Velocity(20.0),
            Velocity(30.0),
            Velocity(40.0),
        ];
        let test_sensor = TestSensor::new(SensorName::Velocity, data_list.clone());

        let mut manager = SensorManager::new();
        manager.add_sensor(test_sensor);
        manager.start_listening_to_sensors();

        let mut index = 0;
        while let Ok(data) = manager.get_data_receiver().recv() {
            assert_eq!(data_list[index], data.data);
            index += 1;
            if index == data_list.len() {
                break;
            }
        }

        assert_eq!(index, data_list.len());
    }
}
