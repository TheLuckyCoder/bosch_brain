#![allow(dead_code)]

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

mod serial;
mod server;
mod track;

fn main() {
    std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().compact())
        .with(EnvFilter::from_default_env())
        .init();
}
