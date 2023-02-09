use serde::Deserialize;
use tokio::net::UdpSocket;

use crate::serial;
use crate::serial::Message;

#[derive(Debug, Clone, Copy, Deserialize)]
struct SteeringWheelData {
    acceleration_percentage: f32,
    break_percentage: f32,
    clutch_percentage: f32,
    steering_angle: f32,
}

pub async fn run_steering_wheel_server() {
    let udp_socket = UdpSocket::bind("10.1.0.10:40000").await.unwrap();

    loop {
        let mut buffer = [0; 4096];
        let size = udp_socket.recv(&mut buffer).await.unwrap();

        let data: SteeringWheelData = serde_json::from_slice(&buffer[0..size]).unwrap();
        println!("{data:?}");

        serial::send(Message::Steer(data.steering_angle));

        let speed_percentage = if data.break_percentage > 0.05 {
            0.0
        } else {
            data.acceleration_percentage - data.clutch_percentage
        };

        serial::send(Message::Speed(speed_percentage * 0.2));
    }
}
