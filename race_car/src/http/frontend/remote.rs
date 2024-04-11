use crate::http::GlobalState;
use askama::Template;
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::{get, put};
use axum::Router;
use axum_extra::{headers, TypedHeader};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

pub fn remote_router(global_state: Arc<GlobalState>) -> Router {
    Router::new()
        .route("/remote", get(get_remote))
        .route("/remote_ws", get(remote_ws))
        .with_state(global_state)
}

#[derive(Template)]
#[template(path = "pages/remote.html")]
struct RemoteTemplate {}

async fn get_remote() -> impl IntoResponse {
    RemoteTemplate {}
}

pub async fn remote_ws(
    State(state): State<Arc<GlobalState>>,
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    info!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    // ws.on_upgrade(move |socket| crate::http::frontend::sensors::handle_socket(socket, addr, state))
}
