use crate::math::AlmostEquals;
use serde::Deserialize;
use tokio::net::UdpSocket;

use crate::serial;
use crate::serial::Message;

#[derive(Debug, Clone, Copy, Deserialize)]
struct SteeringWheelData {
    acceleration_percentage: f32,
    clutch_percentage: f32,
    steering_angle: f32,
}

pub async fn run_steering_wheel_server() {
    let udp_socket = UdpSocket::bind("10.1.0.200:40000").await.unwrap();

    let mut last_speed_percentage = 0.0;
    let mut last_steer = 0.0;

    loop {
        let mut buffer = [0; 4096];
        let size = udp_socket.recv(&mut buffer).await.unwrap();

        let data: SteeringWheelData = serde_json::from_slice(&buffer[0..size]).unwrap();
        // println!("Steering {:?}", data);

        // Do not send the same message twice
        if !last_steer.almost_equals(data.steering_angle, 3.0) {
            // println!("{:?}", data.steering_angle);
            serial::send_blocking(Message::Steer(data.steering_angle)).unwrap();
            last_steer = data.steering_angle;
        }
            // print!("Else ");
            let speed_percentage = data.acceleration_percentage - data.clutch_percentage;

            if !last_speed_percentage.almost_equals(speed_percentage, 0.01) {
                if speed_percentage.abs() < 0.05 {
                    println!("Stopping");
                    serial::send_blocking(Message::Speed(0.0)).unwrap();
                    // last_speed_ = 0.0;
                } else {
                    println!("Driving");
                    let speed = speed_percentage * 0.1;

                    let min_speed = if speed_percentage > 0.0 { 0.1 } else { -0.1 };

                    let final_speed = speed + min_speed;


                    // println!("{:?}", final_speed);
                    serial::send_blocking(Message::Speed(final_speed)).unwrap();

                }
            }

            last_speed_percentage = speed_percentage;

    }
}
