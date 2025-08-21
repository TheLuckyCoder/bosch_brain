//! HTTP routes for controlling the car's PIDs.
use std::sync::Arc;
use std::thread::JoinHandle;

use axum::extract::{Path, Query, State};
use axum::Router;
use axum::routing::post;
use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::{error, info};

use sensors::SensorData;
use shared::math::pid::PidController;

use crate::actuators::ActuatorName;
use crate::http::GlobalState;

/// Holds the PID controllers for the car.
#[derive(Default)]
pub struct PidManager {
    pub velocity: Mutex<Option<(ActuatorName, PidController)>>,
    pub steering: Mutex<Option<(ActuatorName, PidController)>>,
    pub acceleration_thread: Mutex<Option<JoinHandle<()>>>,
}

impl PidManager {
    pub async fn reset(&self) {
        if let Some((_, pid)) = self.velocity.lock().await.as_mut() {
            pid.reset();
        }
        if let Some((_, pid)) = self.steering.lock().await.as_mut() {
            pid.reset();
        }
        *self.acceleration_thread.lock().await = None;
    }
}

/// Creates an object that manages all the PID routes
pub fn router() -> Router<Arc<GlobalState>> {
    Router::new()
        // .route("/", post(set_control_data))
        .route("/velocity_pid/coeff/:actuator", post(velocity_pid_coeff))
        .route("/velocity_pid/target/:value", post(velocity_pid))
        .route("/steering_pid/coeff/:actuator", post(steering_pid_coeff))
        .route("/steering_pid/target/:value", post(steering_pid))
}

/*#[derive(Debug, Deserialize)]
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
    const MAX_HEADING_ANGLE: f64 = 90.0;
    const HEADING_ERROR_WEIGHT: f64 = 0.75;
    const LATERAL_OFFSET_WEIGHT: f64 = 1.35;

    // info!("{:?}", data);
    let mut motor_driver = match state.motor_driver.try_lock() {
        Ok(motor) => motor,
        Err(_) => return,
    };

    if let Some(heading_error) = data.heading_error_degrees {
        let lateral_error = data.lateral_error.unwrap_or_default().clamp(-1.0, 1.0);

        // 1. Normalize heading value
        let normalized_heading_error = heading_error / MAX_HEADING_ANGLE;

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

        info!("Heading: {:.03}; Lateral {lateral_error:.03}; Input: {corrected_heading_error:.03}; Output {pid_output:.03}", normalized_heading_error * HEADING_ERROR_WEIGHT);

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
}*/

/// Sets the target value for the acceleration PID controller.
async fn velocity_pid(State(state): State<Arc<GlobalState>>, Path(target_velocity): Path<f64>) {
    let mut thread = state.pids.acceleration_thread.lock().await;
    if thread.is_none() {
        let receiver = state
            .sensor_manager
            .lock()
            .await
            .get_data_receiver()
            .add_stream();

        let pids = state.pids.clone();
        let actuator_manager = state.actuator_manager.clone();

        let _ = thread.insert(std::thread::spawn(move || loop {
            let mut current_velocity = None;

            while let Ok(sensor_data) = receiver.try_recv() {
                if let SensorData::MotorRPS(velocity) = sensor_data.data {
                    current_velocity = Some(velocity)
                }
            }

            if let Some(velocity) = current_velocity {
                let mut guard = pids.velocity.blocking_lock();
                let Some((actuator_name, pid)) = guard.as_mut() else {
                    break;
                };
                let value = pid.compute(velocity);

                info!("Setting Motor Value: {value}");
                actuator_manager
                    .get_actuator_ref(*actuator_name)
                    .unwrap()
                    .lock()
                    .unwrap()
                    .set_command(value);
            }
        }));
    }
    drop(thread);

    let mut guard = state.pids.velocity.lock().await;
    let Some((_, pid)) = guard.as_mut() else {
        error!("Velocity Pid has not been set up");
        return;
    };
    pid.target_value = target_velocity;
}

#[derive(Deserialize)]
struct PidCoeff {
    p: f64,
    i: f64,
    d: f64,
}

async fn velocity_pid_coeff(
    State(state): State<Arc<GlobalState>>,
    Query(coeff): Query<PidCoeff>,
    Path(actuator_name): Path<ActuatorName>,
) {
    let mut guard = state.pids.velocity.lock().await;
    let _ = guard.insert((actuator_name, PidController::new(coeff.p, coeff.i, coeff.d)));
}

async fn steering_pid_coeff(
    State(state): State<Arc<GlobalState>>,
    Query(coeff): Query<PidCoeff>,
    Path(actuator_name): Path<ActuatorName>,
) {
    let mut guard = state.pids.steering.lock().await;
    let _ = guard.insert((actuator_name, PidController::new(coeff.p, coeff.i, coeff.d)));
}

/// Sets the target value for the steering PID controller.
async fn steering_pid(State(state): State<Arc<GlobalState>>, Path(angle): Path<f64>) {
    let mut guard = state.pids.velocity.blocking_lock();
    let Some((actuator_name, pid)) = guard.as_mut() else {
        return;
    };

    let motor_value = pid.compute((-angle / 30.0f64).clamp(-1.0, 1.0));

    info!("Receiving steering: {angle} ; Motor Value: {motor_value}");
    state
        .actuator_manager
        .get_actuator_ref(*actuator_name)
        .unwrap()
        .lock()
        .unwrap()
        .set_command(motor_value);
}
