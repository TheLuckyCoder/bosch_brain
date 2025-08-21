use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::actuators::{Actuator, ActuatorName};

pub struct ActuatorManager {
    actuators: BTreeMap<ActuatorName, Arc<Mutex<dyn Actuator>>>,
}

impl ActuatorManager {
    pub fn new() -> Self {
        Self {
            actuators: Default::default(),
        }
    }

    pub fn add_actuator(&mut self, actuator: impl Actuator) {
        self.actuators
            .insert(actuator.name(), Arc::new(Mutex::new(actuator)));
    }

    pub fn get_actuator(
        &self,
        actuator_name: ActuatorName,
    ) -> Option<Arc<Mutex<dyn Actuator>>> {
        self.actuators.get(&actuator_name).cloned()
    }

    pub fn get_actuator_ref(
        &self,
        actuator_name: ActuatorName,
    ) -> Option<&Mutex<dyn Actuator>> {
        self.actuators
            .get(&actuator_name)
            .map(|actuator| actuator.as_ref())
    }

    pub fn get_active_actuators(&self) -> Vec<(ActuatorName, &Mutex<dyn Actuator>)> {
        self.actuators
            .iter()
            .map(|(name, actuator)| (*name, actuator.as_ref()))
            .collect()
    }
}
