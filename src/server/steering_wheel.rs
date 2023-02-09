use crate::serial;
use crate::serial::Message;
use serde::Deserialize;
use tokio::net::UdpSocket;

#[derive(Debug, Clone, Copy, Deserialize)]
struct SteeringWheelData {
    acceleration_percentage: Option<f32>,
    break_percentage: Option<f32>,
    clutch_percentage: Option<f32>,
    steering_angle: Option<f32>,
}

pub async fn run_steering_wheel_server() {
    let udp_socket = UdpSocket::bind("10.1.0.10:40000").await.unwrap();

    let mut speed_percentage: f32 = 0_f32;

    loop {
        let mut buffer = [0; 4096];
        let size = udp_socket.recv(&mut buffer).await.unwrap();

        let data: SteeringWheelData = serde_json::from_slice(&buffer[0..size]).unwrap();
        println!("{:?}", data);

        data.steering_angle
            .map(|angle| serial::send(Message::Steer(angle)));

        if data.break_percentage.is_some() && data.break_percentage.unwrap() > 0.05 {
            speed_percentage = 0.0;
        } else {
            let cp = data.clutch_percentage;
            let ap = data.acceleration_percentage;

            match (&ap, &cp) {
                (Some(ap), Some(cp)) => {
                    speed_percentage = ap - cp;
                }
                _ => {}
            }
        }

        serial::send(Message::Speed(speed_percentage * 0.2));
    }
}
