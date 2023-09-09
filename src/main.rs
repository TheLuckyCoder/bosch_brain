#![allow(dead_code)]

use std::io::{BufRead, Read, Write};
use std::net::TcpListener;

use env_logger::Env;

mod brain;
mod math;
mod new_brain;
mod serial;
mod server;
#[cfg(test)]
mod tests;
mod track;

#[derive(serde::Deserialize)]
enum TcpMessage {
    // GetState,
    SetState(String),
    DoCalibration,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format_timestamp(None)
        .target(env_logger::Target::Stdout)
        .init();

    let listener = TcpListener::bind("192.168.0.1:12345").map_err(|e| e.to_string())?;

    let mut buffer = String::with_capacity(128);
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                stream.read_to_string(&mut buffer).unwrap();
                let tcp_message: TcpMessage = serde_json::from_str(&buffer).unwrap();
                match tcp_message {
                    /*TcpMessage::GetState => {
                        stream
                            .write_all(state_machine.get_state().to_string().as_bytes())
                            .unwrap();
                    }*/
                    TcpMessage::SetState(new_state) => {
                        // state_machine.set_state(State::from_str(new_state))
                    }
                    TcpMessage::DoCalibration => {}
                };
            }
            Err(err) => log::error!("{err}"),
        }
    }

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
