use std::sync::Arc;

use askama::Template;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use strum::IntoEnumIterator;

use crate::http::frontend::sensors::{get_sensors, sensor_data_ws_handler};
use crate::http::GlobalState;
use crate::sensors::SensorName;

mod sensors;

pub fn router(global_state: Arc<GlobalState>) -> Router {
    Router::new()
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .route("/", get(get_home))
        .route("/sensors", get(get_sensors))
        .route("/sensor_data", get(sensor_data_ws_handler))
        .with_state(global_state)
}

struct HomeSensor {
    name: &'static str,
    active: bool,
    icon_path: String,
}

#[derive(Template)]
#[template(path = "pages/home.html")]
struct HomeTemplate {
    state: &'static str,
    sensors: Vec<HomeSensor>,
}

async fn get_home(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    let car_state = *state.car_state.lock().await;
    let sensor_manager = state.sensor_manager.lock().await;

    let sensors: Vec<_> = SensorName::iter()
        .map(|sensor_name| {
            let active = sensor_manager.get_sensor(&sensor_name).is_some();
            HomeSensor {
                name: sensor_name.into(),
                active,
                icon_path: format!(
                    "assets/icons/{}_{}.png",
                    sensor_name.to_string().to_lowercase(),
                    if active { "active" } else { "inactive" }
                ),
            }
        })
        .collect();

    HomeTemplate {
        state: car_state.into(),
        sensors,
    }
}


