#![allow(dead_code)]

use crate::math::{AngleWrap, Car, Circle, Point};
use env_logger::Env;
use tokio::task;

use crate::serial::Message;
use crate::server::run_server_listeners;

use plotly::{Layout, Plot, Scatter};
use std::env;
use std::f64::consts::PI;
use std::io::{stdout, Write};
use std::time::Duration;

mod math;
mod serial;
mod server;
#[cfg(test)]
mod tests;
mod track;
mod tui;

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        serial::send_blocking(Message::Speed(0_f32)).expect("Failed to force stop car");
        println!("Car stopped");
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // let _cleanup = Cleanup;

    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format_timestamp(None)
        .target(env_logger::Target::Stdout)
        .init();

    // let tui = task::spawn(tui::run());

    // let track = track::get_track();

    // let imu = sensors::get_imu().expect("Failed to initialize IMU");

    task::spawn(run_server_listeners());
    // tui.await??; // if the TUI task is finished, the program should exit

    let circle_center = Point::new(0.0, 0.0);

    let mut car = Car::new(1 as f64, 0 as f64, PI / 2 as f64, 0.05 as f64, 0 as f64);

    let mut vec_x = Vec::new();

    let mut vec_y = Vec::new();

    let mut c = 0;

    let distance_to_circle = Point::from(&car.position).distance_to(circle_center);

    loop {
        c += 1;

        if c > 250 {
            break;
        }

        vec_x.push(car.position.x);
        vec_y.push(car.position.y);

        let change_of_heading_angle = car.speed / distance_to_circle;

        println!("steering_angle: {:.3}", car.steering_angle.to_degrees());

        // Update the vehicle's position and heading based on the steering angle
        car.position.heading_angle += change_of_heading_angle;

        car.position.heading_angle = car.position.heading_angle.angle_wrap();

        let change_of_x = car.speed * car.position.heading_angle.cos();

        let change_of_y = car.speed * car.position.heading_angle.sin();

        car.position.x += change_of_x;
        car.position.y += change_of_y;

        // Sleep for a small amount of time before updating the position again
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let trace1 = Scatter::new(
        vec![circle_center.x, 1.0, 0.0, -1.0, 0.0],
        vec![circle_center.y, 0.0, 1.0, 0.0, -1.0],
    )
    .name("trace1");

    let trace3 = Scatter::new(vec_x, vec_y).name("trace3");

    let mut plot = Plot::new();
    plot.add_trace(trace1);
    plot.add_trace(trace3);

    let layout = Layout::new().title("<b>Line and Scatter Plot</b>".into());
    plot.set_layout(layout);

    plot.show();

    Ok(())
}
