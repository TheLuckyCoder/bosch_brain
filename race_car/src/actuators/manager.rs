use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::actuators::{ActuatorDriver, ActuatorName};

pub struct ActuatorManager {
    actuators: BTreeMap<ActuatorName, Arc<Mutex<dyn ActuatorDriver>>>,
}

impl ActuatorManager {
    pub fn new() -> Self {
        Self {
            actuators: Default::default(),
        }
    }

    pub fn add_actuator(&mut self, actuator: impl ActuatorDriver) {
        self.actuators
            .insert(actuator.name(), Arc::new(Mutex::new(actuator)));
    }

    pub fn get_actuator(
        &self,
        actuator_name: ActuatorName,
    ) -> Option<Arc<Mutex<dyn ActuatorDriver>>> {
        self.actuators.get(&actuator_name).cloned()
    }

    pub fn get_actuator_ref(
        &self,
        actuator_name: ActuatorName,
    ) -> Option<&Mutex<dyn ActuatorDriver>> {
        self.actuators
            .get(&actuator_name)
            .map(|actuator| actuator.as_ref())
    }

    pub fn get_active_actuators(&self) -> Vec<(ActuatorName, &Mutex<dyn ActuatorDriver>)> {
        self.actuators
            .iter()
            .map(|(name, actuator)| (*name, actuator.as_ref()))
            .collect()
    }
}
