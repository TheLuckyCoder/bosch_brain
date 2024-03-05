//! HTTP routes for controlling the car's PIDs.

use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use tokio::sync::{Mutex, MutexGuard, TryLockError};
use tracing::info;

use shared::math::pid::PidController;

use crate::http::GlobalState;
use crate::sensors::motor_driver::{Motor, MotorDriver};
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
        .route("/", post(set_control_data))
        .route("/velocity_pid/:value", post(velocity_pid))
        .route("/steering_pid/:value", post(steering_pid))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
enum ControlAction {
    LaneKeeping,
    Pause,
    Pause3Seconds,
    Resume,
}

#[derive(Debug, Deserialize)]
struct ControlData {
    heading_error_degrees: Option<f64>,
    observed_acceleration: f64,
    action: ControlAction,
}

async fn set_control_data(State(state): State<Arc<GlobalState>>, Json(data): Json<ControlData>) {
    info!("{:?}", data);
    let mut motor_driver = match state.motor_driver.try_lock() {
        Ok(motor) => motor,
        Err(_) => return,
    };

    match data.action {
        ControlAction::LaneKeeping => {
            // let mut pid = state.pids.steering.lock().await;
            // let angle = pid.compute(-value);
            if let Some(heading_error) = data.heading_error_degrees {
                let motor_value = (-heading_error / 30.0f64).clamp(-1.0, 1.0);

                motor_driver.set_motor_value(Motor::Steering, motor_value);
            }
        }
        ControlAction::Pause => motor_driver.pause_motor(Motor::Speed),
        ControlAction::Pause3Seconds => {
            motor_driver.pause_motor(Motor::Speed);

            let motor_driver = state.motor_driver.clone();
            std::thread::spawn(move || {
                let mut motor_driver = motor_driver.blocking_lock();
                std::thread::sleep(Duration::from_secs(3));
                motor_driver.resume_motor(Motor::Speed);
            });
        }
        ControlAction::Resume => motor_driver.resume_motor(Motor::Speed),
    }
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
