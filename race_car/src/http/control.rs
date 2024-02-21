//! HTTP routes for controlling the car's PIDs.

use std::sync::Arc;
use std::thread::JoinHandle;

use axum::extract::{Path, State};
use axum::routing::post;
use axum::Router;
use tokio::sync::Mutex;
use tracing::info;

use shared::math::pid::PidController;

use crate::http::GlobalState;
use crate::sensors::motor_driver::Motor;
use crate::sensors::SensorData;

/// Holds the PID controllers for the car.
pub struct PidManager {
    pub acceleration: Mutex<PidController>,
    pub steering: Mutex<PidController>,
    pub acceleration_thread: Mutex<Option<JoinHandle<()>>>,
}

impl PidManager {
    pub async fn reset(&self) {
        self.acceleration.lock().await.reset();
        self.steering.lock().await.reset();
        *self.acceleration_thread.lock().await = None;
    }
}

impl PidManager {
    pub fn new(acceleration: PidController, steering: PidController) -> Self {
        Self {
            acceleration: Mutex::new(acceleration),
            steering: Mutex::new(steering),
            acceleration_thread: Mutex::default(),
        }
    }
}

/// Creates an object that manages all the PID routes
pub fn router(state: Arc<GlobalState>) -> Router {
    Router::new()
        .route("/velocity_pid/:value", post(velocity_pid))
        .route("/steering_pid/:value", post(steering_pid))
        .with_state(state)
}

/// Sets the target value for the acceleration PID controller.
async fn velocity_pid(State(state): State<Arc<GlobalState>>, Path(target_velocity): Path<f64>) {
    {
        let mut thread = state.pids.acceleration_thread.lock().await;
        if thread.is_none() {
            let receiver = state
                .sensor_manager
                .lock()
                .await
                .get_data_receiver()
                .add_stream();

            let pids = state.pids.clone();
            let motor_driver = state.motor_driver.clone();

            let _ = thread.insert(std::thread::spawn(move || loop {
                let mut current_velocity = None;

                while let Ok(sensor_data) = receiver.try_recv() {
                    if let SensorData::Velocity(velocity) = sensor_data.data {
                        current_velocity = Some(velocity)
                    }
                }

                if let Some(velocity) = current_velocity {
                    let value = {
                        let mut pid = pids.acceleration.blocking_lock();
                        pid.compute(velocity)
                    };

                    info!("Setting Motor Value: {value}");
                    motor_driver
                        .blocking_lock()
                        .set_motor_value(Motor::Speed, value);
                }
            }));
        }
    }

    let mut pid = state.pids.acceleration.lock().await;
    pid.target_value = target_velocity;
}

/// Sets the target value for the steering PID controller.
async fn steering_pid(State(state): State<Arc<GlobalState>>, Path(angle): Path<f64>) {
    let mut motor = state.motor_driver.lock().await;

    // let mut pid = state.pids.steering.lock().await;
    // let angle = pid.compute(-value);
    let motor_value = (-angle / 30.0f64).clamp(-1.0, 1.0);

    info!("Receiving steering: {angle} ; Motor Value: {motor_value}");
    motor.set_motor_value(Motor::Steering, motor_value);
}
