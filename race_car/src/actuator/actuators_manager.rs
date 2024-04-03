use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::actuator::ActuatorName;
use crate::sensors::{BasicSensor, SensorName};

struct ActuatorsManager {
    sensors: HashMap<ActuatorName, Box<Mutex<dyn BasicSensor + Send>>>,
}
