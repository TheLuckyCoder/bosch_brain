use std::sync::Arc;

use ::sensors::name::SensorName;
use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::Router;
use strum::IntoEnumIterator;
use tower_livereload::LiveReloadLayer;

use crate::http::config::ServerConfig;
use crate::http::GlobalState;
use crate::sensors::add_all_sensors;
use crate::sensors::mock::add_all_mock_sensors;

mod actuators;
mod remote;
mod sensors;

pub fn router() -> Router<Arc<GlobalState>> {
    Router::new()
        .nest_service("/assets", tower_http::services::ServeDir::new("assets"))
        .route("/", get(get_home))
        .nest("/sensors", sensors::sensors_router())
        .nest("/remote", remote::remote_router())
        .nest("/actuators", actuators::actuators_router())
        // .layer(LiveReloadLayer::new())
        .route("/config/mock_sensors", post(toggle_mock_sensors))
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
    server_config: ServerConfig,
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
        server_config: state.server_config.lock().await.clone(),
    }
}

async fn toggle_mock_sensors(State(state): State<Arc<GlobalState>>) -> impl IntoResponse {
    let mut server_config = state.server_config.lock().await;
    let mut sensor_manager = state.sensor_manager.lock().await;

    server_config.mock_sensors = !server_config.mock_sensors;
    server_config.save_to_file().unwrap();

    sensor_manager.remove_all_sensors();
    if server_config.mock_sensors {
        add_all_mock_sensors(&mut sensor_manager);
    } else {
        add_all_sensors(&mut sensor_manager);
    }

    (StatusCode::OK, [("HX-Refresh", "true")])
}
