//! HTTP routes for controlling the car's PIDs.

use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
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
        .route("/", post(set_control_data))
        .route("/velocity_pid/:value", post(velocity_pid))
        .route("/steering_pid", get(steering_pid_coeff))
        .route("/steering_pid/:value", post(steering_pid))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
enum ControlAction {
    LaneKeeping,
    Pause,
    Pause3Seconds,
    Resume,
    RightTurn,
    LeftTurn,
}

#[derive(Debug, Deserialize)]
struct ControlData {
    heading_error_degrees: Option<f64>,
    lateral_error: Option<f64>,
    observed_acceleration: f64,
    action: ControlAction,
}

async fn set_control_data(State(state): State<Arc<GlobalState>>, Json(data): Json<ControlData>) {
    const MAX_MOTOR_DEGREES: f64 = 30.0;
    const HEADING_ERROR_WEIGHT: f64 = 0.2;
    const LATERAL_OFFSET_WEIGHT: f64 = 1.2;

    // info!("{:?}", data);
    let mut motor_driver = match state.motor_driver.try_lock() {
        Ok(motor) => motor,
        Err(_) => return,
    };

    if let Some(heading_error) = data.heading_error_degrees {
        let lateral_error = data.lateral_error.unwrap_or_default();

        // 1. Normalize heading value
        let normalized_heading_error = heading_error / MAX_MOTOR_DEGREES;

        // 2. Lateral Offset Correction
        // positive value means car is on the right side of the road
        // new_heading_error = heading_error + k * lateral_offset
        let corrected_heading_error =
            HEADING_ERROR_WEIGHT * normalized_heading_error + LATERAL_OFFSET_WEIGHT * lateral_error;

        // 3. Run PID
        let pid_output = state
            .pids
            .steering
            .lock()
            .await
            .compute(corrected_heading_error);

        info!("Heading: {heading_error:.03}; Lateral {lateral_error:.03}; Input: {corrected_heading_error:.03}; Output {pid_output:.03}");

        let car_direction = motor_driver.get_last_motor_value(Motor::Speed).signum();
        motor_driver.set_motor_value(Motor::Steering, car_direction * pid_output);
    }

    match data.action {
        ControlAction::LaneKeeping => {}
        ControlAction::Pause => motor_driver.pause_motor(Motor::Speed),
        ControlAction::Pause3Seconds => {
            info!("Taking 3 seconds pause");
            let current_speed = motor_driver.get_last_motor_value(Motor::Speed);
            motor_driver.pause_motor(Motor::Speed);

            let motor_driver = state.motor_driver.clone();
            std::thread::spawn(move || {
                let mut motor_driver = motor_driver.blocking_lock();
                std::thread::sleep(Duration::from_secs(3));
                motor_driver.resume_motor(Motor::Speed);
                motor_driver.set_motor_value(Motor::Speed, current_speed);
                info!("Finished taking 3 seconds pause");
            });
        }
        ControlAction::Resume => motor_driver.resume_motor(Motor::Speed),
        ControlAction::RightTurn => {}
        ControlAction::LeftTurn => {}
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

#[derive(Deserialize)]
struct PidCoeff {
    p: f64,
    i: f64,
    d: f64,
}

async fn steering_pid_coeff(State(state): State<Arc<GlobalState>>, Query(coeff): Query<PidCoeff>) {
    let mut pid = state.pids.steering.lock().await;
    pid.k_p = coeff.p;
    pid.k_i = coeff.i;
    pid.k_d = coeff.d;
    pid.reset();
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
