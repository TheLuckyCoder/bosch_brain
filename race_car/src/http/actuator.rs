//! HTTP routes for manually controlling the car's motors.
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use strum::IntoEnumIterator;

use crate::actuators::ActuatorName;
use crate::http::GlobalState;

/*fn get_actuator_params_file(actuator: Motor) -> PathBuf {
    let mut path = get_car_dir();
    path.push(&format!("motor_params_{actuator:?}.json"));
    path
}

async fn read_params_from_files(global_state: &Arc<GlobalState>) {
    for actuator in ALL_actuatorS {
        let file_path = get_actuator_params_file(actuator);
        let params_file = match std::fs::File::open(&file_path) {
            Ok(params_file) => params_file,
            Err(e) => {
                log::warn!("Failed to read {} reason: {e}", file_path.display());
                continue;
            }
        };
        let mut reader = BufReader::new(params_file);

        match serde_json::from_reader(&mut reader) {
            Ok(params) => global_state
                .motor_driver
                .lock()
                .await
                .set_params(actuator, params),
            Err(e) => {
                log::error!("Failed to deserialize {} reason: {e}", file_path.display());
            }
        }
    }
}*/

/// Creates an object that manages all the actuator routes
pub fn router() -> Router<Arc<GlobalState>> {
    Router::new()
        .route("/", get(get_actuators))
        .route("/stop/:actuator", post(stop_actuator))
        .route("/stop", post(stop_actuator))
        .route("/set/:actuator/:value", post(set_actuator_value))
        .route("/pause/:actuator", post(pause_actuator))
        .route("/pause", post(pause_actuator))
        .route("/resume/:actuator", post(resume_actuator))
        .route("/resume", post(resume_actuator))
}

/// Returns a list of all motors
async fn get_actuators() -> impl IntoResponse {
    Json(ActuatorName::iter().collect::<Vec<_>>())
}

/// Returns the current parameters for the given actuator
/*async fn get_actuator_parameters(
    State(state): State<Arc<GlobalState>>,
    Path(actuator): Path<ActuatorName>,
) -> impl IntoResponse {
    let motor_driver = state.motor_driver.lock().await;

    Json(motor_driver.get_params(actuator))
}*/

/// Sets the parameters for the given actuator
/*async fn set_actuator_parameters(
    State(state): State<Arc<GlobalState>>,
    Path(actuator): Path<ActuatorName>,
    Json(params): Json<MotorParams>,
) {
    let mut motor_driver = state.motor_driver.lock().await;

    info!("Motor params received: {params:?}");
    motor_driver.set_params(actuator, params.clone());

    let motor_params_path = get_actuator_params_file(actuator);

    task::spawn_blocking(move || {
        let file = std::fs::File::create(motor_params_path).unwrap();
        let mut writer = BufWriter::new(file);

        serde_json::to_writer(&mut writer, &params).unwrap();
        writer.flush().unwrap();
        writer.get_ref().sync_all().unwrap();
    })
    .await
    .unwrap();
}*/

/// Stops the given actuator, or all motors if no actuator is specified
async fn stop_actuator(
    State(state): State<Arc<GlobalState>>,
    actuator: Option<Path<ActuatorName>>,
) {
    if let Some(Path(actuator)) = actuator {
        state
            .actuator_manager
            .get_actuator_ref(actuator)
            .unwrap()
            .lock()
            .unwrap()
            .stop();
    } else {
        for actuator in ActuatorName::iter() {
            if let Some(actuator) = state.actuator_manager.get_actuator_ref(actuator) {
                actuator.lock().unwrap().stop();
            }
        }
    }
}

/// Sets the value for the given actuator
async fn set_actuator_value(
    State(state): State<Arc<GlobalState>>,
    Path((actuator, value)): Path<(ActuatorName, f64)>,
) {
    state
        .actuator_manager
        .get_actuator_ref(actuator)
        .unwrap()
        .lock()
        .unwrap()
        .set_value(value);
}

/// Pauses the given actuator, or all motors if no actuator is specified
async fn pause_actuator(
    State(state): State<Arc<GlobalState>>,
    actuator: Option<Path<ActuatorName>>,
) {
    if let Some(Path(actuator)) = actuator {
        state
            .actuator_manager
            .get_actuator_ref(actuator)
            .unwrap()
            .lock()
            .unwrap()
            .pause();
    } else {
        for actuator in ActuatorName::iter() {
            if let Some(actuator) = state.actuator_manager.get_actuator_ref(actuator) {
                actuator.lock().unwrap().pause();
            }
        }
    }
}

/// Resumes the given actuator, or all motors if no actuator is specified
async fn resume_actuator(
    State(state): State<Arc<GlobalState>>,
    actuator: Option<Path<ActuatorName>>,
) {
    if let Some(Path(actuator)) = actuator {
        state
            .actuator_manager
            .get_actuator_ref(actuator)
            .unwrap()
            .lock()
            .unwrap()
            .resume();
    } else {
        for actuator in ActuatorName::iter() {
            if let Some(actuator) = state.actuator_manager.get_actuator_ref(actuator) {
                actuator.lock().unwrap().resume();
            }
        }
    }
}

/*async fn motor_sweep(State(state): State<Arc<GlobalState>>, Path(actuator): Path<ActuatorName>) {
    if *state.car_state.lock().await != CarStates::Config {
        return;
    }

    tokio::spawn(async move {
        let mut motor_driver = state.motor_driver.lock().await;

        for i in 0..10 {
            motor_driver.set_actuator_value(actuator, i as f64 / 10f64);
            sleep(Duration::from_millis(150)).await;
        }

        for i in -10..=10 {
            motor_driver.set_actuator_value(actuator, -i as f64 / 10f64);
            sleep(Duration::from_millis(150)).await;
        }
        for i in 0..=10 {
            motor_driver.set_actuator_value(actuator, (-10 + i) as f64 / 10f64);
            sleep(Duration::from_millis(150)).await;
        }

        motor_driver.stop_actuator(actuator);
    });
}*/
