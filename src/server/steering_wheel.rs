use crate::math::AlmostEquals;
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

    let mut last_speed = 0.0;
    let mut last_steer = 0.0;

    loop {
        let mut buffer = [0; 4096];
        let size = udp_socket.recv(&mut buffer).await.unwrap();

        let data: SteeringWheelData = serde_json::from_slice(&buffer[0..size]).unwrap();
        println!("{data:?}");

        // Do not send the same message twice
        if !last_steer.almost_equals(data.steering_angle, 0.01) {
            serial::send_blocking(Message::Steer(data.steering_angle)).unwrap();
            last_steer = data.steering_angle;
        }

        let speed_percentage = if data.break_percentage > 0.05 {
            0.0
        } else {
            data.acceleration_percentage - data.clutch_percentage
        };

        let speed = speed_percentage * 0.2;
        if !last_speed.almost_equals(speed, 0.01) {
            serial::send_blocking(Message::Speed(speed)).unwrap();
            last_speed = speed;
        }
    }
}
