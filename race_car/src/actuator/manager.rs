use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::actuator::{Actuator, ActuatorName};

pub struct ActuatorManager {
    sensors: HashMap<ActuatorName, Arc<Mutex<dyn Actuator + Send>>>,
}

impl ActuatorManager {
    pub fn new() -> Self {
        Self {
            sensors: Default::default(),
        }
    }
    
    pub fn add_actuator(&mut self, actuator: impl Actuator) {
        self.sensors.insert(actuator.name(), Arc::new(Mutex::new(actuator)));
    }

    pub fn get_actuator(&self, actuator_name: ActuatorName) -> Option<Arc<Mutex<dyn Actuator + Send>>> {
        self.sensors.get(&actuator_name).cloned()
    }
    
    pub fn get_actuator_ref(&self, actuator_name: ActuatorName) -> Option<&Mutex<dyn Actuator + Send>> {
        self.sensors.get(&actuator_name).map(|actuator| actuator.as_ref())
    }
}
