use std::fmt::Display;

use crossterm::terminal::{Clear, ClearType};
use crossterm::{
    event::{Event, EventStream, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
    Result,
};
use futures::{future::FutureExt, StreamExt};

use crate::serial;
use crate::serial::Message;

#[derive(Debug)]
struct CarParams {
    pub speed: f32,
    pub angle: f32,
    pub pid_enabled: bool,
    pub speed_step: f32,
    pub angle_step: f32,
    pub k_p: f32,
    pub k_i: f32,
    pub k_d: f32,
    pub k_f: f32,
    pub k_p_step: f32,
    pub k_i_step: f32,
    pub k_d_step: f32,
}

impl Display for CarParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Speed: {:.2}\nAngle: {:.2}\nPID Enabled: {}\nSpeed Step: {:.2}\nAngle Step: {:.2}\nKp: {:.5}\nKi: {:.5}\nKd: {:.5}\nKf: {:.5}",
            self.speed, self.angle, self.pid_enabled, self.speed_step, self.angle_step, self.k_p, self.k_i, self.k_d, self.k_f
        )
    }
}

fn print_params(params: &CarParams) {
    execute!(std::io::stdout(), Clear(ClearType::All)).unwrap();
    println!("=========== REMOTE CONTROL ============\n{}", params);
}

async fn print_events() {
    let mut reader = EventStream::new();
    let mut params = CarParams {
        speed: 0.0,
        angle: 0.0,
        pid_enabled: false,
        speed_step: 2.0,
        angle_step: 1.5,
        k_p: 0.11500,
        k_i: 0.81000,
        k_d: 0.00022,
        k_f: 0.04000,
        k_p_step: 0.001,
        k_i_step: 0.001,
        k_d_step: 0.000001,
    };

    loop {
        let event = reader.next().fuse();

        match event.await {
            Some(Ok(event)) => {
                log::debug!("Event::{:?}", event); // TODO Remove

                if let Event::Key(key_event) = event {
                    if react_to_keys(key_event.code, &mut params).unwrap() {
                        break;
                    }
                    print_params(&params);
                }
            }
            Some(Err(e)) => log::error!("Error: {:?}", e),
            None => break,
        }
    }
}

fn react_to_keys(key: KeyCode, params: &mut CarParams) -> std::io::Result<bool> {
    let serial = serial::get_serial();

    match key {
        KeyCode::Char('w') => {
            params.speed += params.speed_step;
            serial.send_blocking(Message::speed(params.speed))?;
        }
        KeyCode::Char('s') => {
            params.speed -= params.speed_step;
            serial.send_blocking(Message::speed(params.speed))?;
        }
        KeyCode::Char('a') => {
            params.angle -= params.angle_step;
            serial.send_blocking(Message::steer(params.angle))?;
        }
        KeyCode::Char('d') => {
            params.angle += params.angle_step;
            serial.send_blocking(Message::steer(params.angle))?;
        }
        KeyCode::Char('b') => {
            serial.send_blocking(Message::brake(params.angle))?;
        }
        KeyCode::Char('p') => {
            params.pid_enabled = !params.pid_enabled;
            serial.send_blocking(Message::enable_pid(params.pid_enabled))?;
            serial.send_blocking(Message::pid_constants(
                params.k_d, params.k_i, params.k_d, params.k_f,
            ))?;
        }
        KeyCode::Char('q') => {
            return Ok(true); // Exit
        }
        _ => {}
    }

    Ok(false)
}

pub async fn run() -> Result<()> {
    enable_raw_mode()?;

    print_events().await;

    disable_raw_mode()
}
