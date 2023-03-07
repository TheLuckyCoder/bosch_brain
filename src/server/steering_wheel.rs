use crate::math::AlmostEquals;
use serde::Deserialize;
use std::time::Instant;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::net::UdpSocket;

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
        .open("commands_history.json")
        .await?;

    let mut last_speed_percentage = 0.0;
    let mut last_steer = 0.0;
    let mut last_message_instant = Instant::now();

    loop {
        let mut buffer = [0; 4096];
        let size = udp_socket.recv(&mut buffer).await?;

        let data: SteeringWheelData = serde_json::from_slice(&buffer[0..size])?;

        // Do not send the same message twice
        if !last_steer.almost_equals(data.steering_angle, 3.0) {
            serial::send_blocking(Message::Steer(data.steering_angle))?;
            last_steer = data.steering_angle;
            file.write_all(
                format!("{}|{}", last_message_instant.elapsed().as_millis(), message).as_bytes(),
            )
            .await?;
            last_message_instant = Instant::now();
        }

        let speed_percentage = data.acceleration_percentage - data.clutch_percentage;

        if !last_speed_percentage.almost_equals(speed_percentage, 0.01) {
            let message = if speed_percentage.abs() < 0.05 {
                // println!("Stopping");
                let message = Message::Speed(0.0);
                serial::send_blocking(message.clone())?;
                message
            } else {
                // println!("Driving");
                let speed = speed_percentage * 0.1;
                let min_speed = if speed_percentage > 0.0 { 0.1 } else { -0.1 };
                let final_speed = speed + min_speed;

                let message = Message::Speed(final_speed);
                serial::send_blocking(message.clone())?;
                message
            };

            file.write_all(
                format!("{}|{}", last_message_instant.elapsed().as_millis(), message).as_bytes(),
            )
            .await?;
            last_message_instant = Instant::now();
        }

        last_speed_percentage = speed_percentage;
    }
}
