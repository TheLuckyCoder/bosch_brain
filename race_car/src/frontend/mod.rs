use crate::http::GlobalState;
use crate::sensors::SensorName;
use askama::Template;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use serde::Serialize;
use std::sync::Arc;
use strum::IntoEnumIterator;

pub fn router(global_state: Arc<GlobalState>) -> Router {
    Router::new()
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .route("/", get(sensors))
        .with_state(global_state)
}

#[derive(Serialize)]
struct Sensor {
    name: &'static str,
    enabled: bool,
}

#[derive(Template)]
#[template(path = "sensors.html")]
struct SensorsTemplate {
    sensors: Vec<Sensor>,
}

async fn sensors(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    let sensor_manager = state.sensor_manager.lock().await;

    let sensors: Vec<_> = SensorName::iter()
        .map(|sensor_name| Sensor {
            name: sensor_name.into(),
            enabled: sensor_manager.get_sensor(&sensor_name).is_some(),
        })
        .collect();

    SensorsTemplate { sensors }
}
