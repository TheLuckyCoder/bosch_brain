use std::fs::OpenOptions;
use std::time::Instant;

use serde::Deserialize;
use std::io::prelude::*;
use tokio::net::UdpSocket;

use crate::math::AlmostEquals;
use crate::serial;
use crate::serial::Message;

#[derive(Debug, Clone, Copy, Deserialize)]
struct SteeringWheelData {
    acceleration_percentage: f32,
    clutch_percentage: f32,
    steering_angle: f32,
}

pub async fn run_steering_wheel_server() -> std::io::Result<()> {
    let udp_socket = UdpSocket::bind("10.1.0.200:40000").await?;

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("commands_history.psv")?;

    let mut last_speed_percentage = 0.0;
    let mut last_steer = 0.0;
    let mut last_message_instant = Instant::now();

    let mut append_message_to_file = |message: &Message| {
        writeln!(
            file,
            "{}|{}",
            last_message_instant.elapsed().as_millis(),
            message.to_string().trim()
        )
        .unwrap();
        last_message_instant = Instant::now();
    };

    loop {
        let mut buffer = [0; 4096];
        let size = udp_socket.recv(&mut buffer).await?;

        let data: SteeringWheelData = serde_json::from_slice(&buffer[0..size])?;

        // Do not send the same message twice
        if !last_steer.almost_equals(data.steering_angle, 3.0) {
            let message = Message::Steer(data.steering_angle);
            last_steer = data.steering_angle;
            append_message_to_file(&message);

            serial::send_blocking(message)?;
        }

        let speed_percentage = data.acceleration_percentage - data.clutch_percentage;

        if !last_speed_percentage.almost_equals(speed_percentage, 0.01) {
            let message = if speed_percentage.abs() < 0.05 {
                // println!("Stopping");
                Message::Speed(0.0)
            } else {
                // println!("Driving");
                let speed = speed_percentage * 0.1;
                let min_speed = if speed_percentage > 0.0 { 0.1 } else { -0.1 };
                let final_speed = speed + min_speed;

                Message::Speed(final_speed)
            };

            append_message_to_file(&message);
            serial::send_blocking(message)?;
        }

        last_speed_percentage = speed_percentage;
    }
}
