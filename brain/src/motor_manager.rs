use crate::math::AlmostEquals;
use crate::utils::atomic_f64::AtomicF64;
use sensors::MotorDriver;
use std::sync::atomic::Ordering;
use std::sync::Mutex;

pub struct MotorManager {
    motor_driver: Mutex<MotorDriver>,
    last_acceleration: AtomicF64,
    last_steering: AtomicF64,
}

impl MotorManager {
    pub fn new() -> Self {
        Self {
            motor_driver: Mutex::new(
                MotorDriver::new()
                    .unwrap_or_else(|e| panic!("Failed to initialize MotorDriver: {e}")),
            ),
            last_acceleration: AtomicF64::default(),
            last_steering: AtomicF64::default(),
        }
    }

    pub fn set_acceleration(&self, acceleration: f64) {
        let last = self.last_acceleration.load(Ordering::AcqRel);

        if !acceleration.almost_equals(last, 10e-2) {
            self.last_acceleration.store(acceleration, Ordering::AcqRel);
            self.motor_driver
                .lock()
                .unwrap()
                .set_acceleration(acceleration);
        }
    }

    pub fn set_steering(&self, steering: f64) {
        let last = self.last_steering.load(Ordering::AcqRel);

        if !steering.almost_equals(last, 10e-2) {
            self.last_steering.store(steering, Ordering::AcqRel);
            self.motor_driver
                .lock()
                .unwrap()
                .set_steering_angle(steering);
        }
    }
}
