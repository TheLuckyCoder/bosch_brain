#![allow(dead_code)]

use std::io::{BufRead, Read, Write};

use env_logger::Env;

mod http;
mod math;
mod motor_manager;
mod serial;
mod server;
#[cfg(test)]
mod tests;
mod track;
mod udp_manager;
mod utils;

#[tokio::main]
async fn main() -> Result<(), String> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format_timestamp(None)
        .target(env_logger::Target::Stdout)
        .init();

    // let track = track::get_track();

    // let path = "/home/car/recorded_movements/full_run.txt";

    // task::spawn(async move {
    //     if let Err(e) = server::steering_wheel::run_steering_wheel_server(path).await {
    //         log::error!("Steering wheel server error: {e}");
    //     }
    // });
    // if Path::new(path).exists() {
    //     let file = OpenOptions::new().read(true).open(path)?;
    //
    //     // read all lines from file and store them in a Vec
    //     let lines: Vec<_> = std::io::BufReader::new(file)
    //         .lines()
    //         .map(|l| l.unwrap())
    //         .collect();
    //
    //     for line in lines {
    //         let mut split = line.split('|');
    //         let time = split.next().unwrap();
    //         let message = split.next().unwrap();
    //
    //         serial::send_blocking(Message::Raw(message.to_string()))?;
    //         //sleep for time milliseconds
    //         sleep(std::time::Duration::from_millis(
    //             time.parse::<u64>().unwrap(),
    //         ));
    //     }
    // }
    // serial::send_blocking(Message::Speed(0_f32))?; //stop car
    // brain::start_brain();

    Ok(())
}
