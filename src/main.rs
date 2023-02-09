#![allow(dead_code)]

use env_logger::Env;
use tokio::task;

use crate::serial::Message;

mod brain;
mod math;
mod serial;
mod server;
#[cfg(test)]
mod tests;
mod track;

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        serial::send_blocking(Message::Speed(0_f32)).expect("Failed to force stop car");
        println!("Car stopped");
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // let _cleanup = Cleanup;

    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format_timestamp(None)
        .target(env_logger::Target::Stdout)
        .init();

    // let track = track::get_track();
    task::spawn(server::steering_wheel::run_steering_wheel_server());

    brain::start_brain();

    Ok(())
}
