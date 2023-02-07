use crate::serial;
use crate::serial::Message;
use serde::Deserialize;
use tokio::net::UdpSocket;

#[derive(Debug, Clone, Copy, Deserialize)]
struct SteeringWheelData {
    speed: Option<f32>,
    steering_angle: Option<f32>,
}

pub async fn run_steering_wheel_server() {
    let udp_socket = UdpSocket::bind("0.0.0.0:40000").await.unwrap();

    loop {
        let mut buffer = [0; 4096];
        let size = udp_socket.recv(&mut buffer).await.unwrap();

        let data: SteeringWheelData = serde_json::from_slice(&buffer[0..size]).unwrap();
        println!("{:?}", data);
        data.speed.map(|speed| serial::send(Message::Speed(speed)));
        data.steering_angle
            .map(|angle| serial::send(Message::Speed(angle)));
    }
}
