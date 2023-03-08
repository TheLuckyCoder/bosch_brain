#![allow(dead_code)]

use env_logger::Env;
use std::fs::OpenOptions;
use std::io::BufRead;
use std::path::Path;
use std::thread::sleep;
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

    if true {
        task::spawn(async move {
            if let Err(e) = server::steering_wheel::run_steering_wheel_server().await {
                log::error!("Steering wheel server error: {e}");
            }
        });
    } else if Path::new("commands_history.psv").exists() {
        let file = OpenOptions::new().read(true).open("commands_history.psv")?;

        // read all lines from file and store them in a Vec
        let lines: Vec<_> = std::io::BufReader::new(file)
            .lines()
            .map(|l| l.unwrap())
            .collect();

        for line in lines {
            let mut split = line.split('|');
            let time = split.next().unwrap();
            let message = split.next().unwrap();

            serial::send_blocking(Message::Raw(message.to_string()))?;
            //sleep for time milliseconds
            sleep(std::time::Duration::from_millis(
                time.parse::<u64>().unwrap(),
            ));
        }
    }

    brain::start_brain();

    Ok(())
}
