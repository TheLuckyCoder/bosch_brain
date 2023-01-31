use std::env;
use std::f64::consts::PI;
use std::io::{stdout, Write};
use std::time::{Duration};
use plotly::{Layout, Plot, Scatter};


use crate::math::{AngleWrap, Car, Circle, Point, Segment};

const MAX_STEERING_ANGLE: f64 = 0.4363323129985824; // 25 degrees

fn pure_pursuit(center_x: f64, center_y: f64, vehicle_x: f64, vehicle_y: f64, vehicle_heading: f64, wheelbase: f64) -> f64 {
    let lookahead_distance = wheelbase;
    let lookahead_x = center_x + lookahead_distance * vehicle_heading.cos();
    let lookahead_y = center_y + lookahead_distance * vehicle_heading.sin();
    let lookahead_heading = (lookahead_y - vehicle_y).atan2(lookahead_x - vehicle_x);
    let steering_angle = vehicle_heading - lookahead_heading;
    steering_angle
}


fn pure_pursuit_closed_loop(center_x: f64, center_y: f64, vehicle_x: f64, vehicle_y: f64, vehicle_heading: f64, vehicle_speed: f64, wheelbase: f64, kp: f64, ki: f64, kd: f64) -> f64 {
    // Calculate the desired path
    let dx = center_x - vehicle_x;
    let dy = center_y - vehicle_y;
    let path_distance = (dx * dx + dy * dy).sqrt();
    let angle_to_path = dy.atan2(dx) - vehicle_heading;

    // Calculate the error
    let error = angle_to_path.tan() * wheelbase / path_distance;

    // Use a PID controller to generate the control signal (the steering angle)
    let proportional = error * kp;
    // let integral = /* code to calculate integral of error */;
    // let derivative = /* code to calculate derivative of error */;
    // let steering_angle = proportional + integral + derivative;
    let steering_angle = proportional;

    // Ensure that the steering angle is within the vehicle's maximum steering angle
    if steering_angle > MAX_STEERING_ANGLE {
        MAX_STEERING_ANGLE
    } else if steering_angle < -MAX_STEERING_ANGLE {
        -MAX_STEERING_ANGLE
    } else {
        steering_angle
    }
}


#[test]
fn follow_track() {
    let circle_center = Point::new(0.0, 0.0);

    let mut car = Car::new(10 as f64, 0 as f64, 0 as f64, 1 as f64, 0 as f64);

    let mut vec_x = Vec::new();

    let mut vec_y = Vec::new();

    let mut c = 0;

    loop {
        c += 1;

        if c > 50 {
            break;
        }

        vec_x.push(car.position.x);
        vec_y.push(car.position.y);

        // Calculate the steering angle
        car.steering_angle = pure_pursuit(circle_center.x, circle_center.y, car.position.x, car.position.y,
                                          car.position.heading_angle, 2.0);

        println!("steering_angle: {:.3}", car.steering_angle.to_degrees());

        // Update the vehicle's position and heading based on the steering angle
        car.position.x += car.speed * car.position.heading_angle.cos();
        car.position.y += car.speed * car.position.heading_angle.sin();
        car.position.heading_angle += car.steering_angle;

        // Sleep for a small amount of time before updating the position again
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let trace1 = Scatter::new(vec![circle_center.x, 1.0, 0.0, -1.0, 0.0], vec![circle_center.y, 0.0, 1.0, 0.0, -1.0])
        .name("trace1");

    let trace3 = Scatter::new(vec_x, vec_y)
        .name("trace3");

    let mut plot = Plot::new();
    plot.add_trace(trace1);
    plot.add_trace(trace3);

    let layout = Layout::new().title("<b>Line and Scatter Plot</b>".into());
    plot.set_layout(layout);

    plot.show();
}

#[test]
fn simulate_car_movement() {
    let circle_center = Point::new(0.0, 0.0);

    let mut car = Car::new(10 as f64, 0 as f64, 0 as f64, 1 as f64, 0 as f64);

    let mut vec_x = Vec::new();

    let mut vec_y = Vec::new();

    let wheelbase = 2.0;

    let radius = ((circle_center.x - car.position.x).powi(2) + (circle_center.y - car.position.y).powi(2)).sqrt();

    let mut error_heading_derivative = 0.0;
    let mut time = 0.0;
    let dt = 0.1;
    let kd_heading = 0.1;
    let kp_heading = 0.1;

    loop {
        // calculate heading error
        let error_heading = car.position.heading_angle - (car.position.heading_angle - (car.speed / radius));
        error_heading_derivative = (error_heading - error_heading_derivative) / dt;

        // PD control for heading
        let heading_control = kp_heading * error_heading + kd_heading * error_heading_derivative;
        let steering_angle = heading_control * (wheelbase / car.speed);

        vec_x.push(car.position.x);
        vec_y.push(car.position.y);

        // update vehicle position and heading
        car.position.x += car.speed * dt * car.position.heading_angle.cos();
        car.position.x += car.speed * dt * car.position.heading_angle.sin();
        car.position.heading_angle += (car.speed / radius) * dt;

        // update time
        time += dt;

        // print vehicle position, heading, and time
        println!("time: {:.1}s, x: {:.2}, y: {:.2}, heading: {:.2}, steering_angle: {:.2}", time, car.position.x, car.position.x, car.position.heading_angle, steering_angle);

        // stop the simulation after a certain amount of time
        if time > 10.0 {
            break;
        }
    }

    let trace1 = Scatter::new(vec![circle_center.x, 1.0, 0.0, -1.0, 0.0], vec![circle_center.y, 0.0, 1.0, 0.0, -1.0])
        .name("trace1");

    let trace3 = Scatter::new(vec_x, vec_y)
        .name("trace3");

    let mut plot = Plot::new();
    plot.add_trace(trace1);
    plot.add_trace(trace3);

    let layout = Layout::new().title("<b>Line and Scatter Plot</b>".into());
    plot.set_layout(layout);

    plot.show();
}

#[test]
fn test_lateral_control() {
    let circle_center = Point::new(0.0, 0.0);

    let mut car = Car::new(10 as f64, 0 as f64, PI / 2 as f64, 2 as f64, 0 as f64);

    let wheelbase = 2.0;

    lateral_control(circle_center.x, circle_center.y, car.position.x, car.position.y, car.position.heading_angle, car.speed, wheelbase);
}

fn lateral_control(center_x: f64, center_y: f64, car_x: f64, car_y: f64, car_heading: f64, speed: f64, wheelbase: f64) {
    let mut vec_x = Vec::new();

    let mut vec_y = Vec::new();

    let radius = ((center_x - car_x).powi(2) + (center_y - car_y).powi(2)).sqrt();
    let mut car_x = car_x;
    let mut car_y = car_y;
    let mut car_heading = car_heading;
    let mut time = 0.0;
    let dt = 0.1;
    loop {
        vec_x.push(car_x);
        vec_y.push(car_y);


        let desired_yaw_rate = speed / radius;

        // Steering angle
        let steering_angle = desired_yaw_rate * wheelbase / speed;

        // Update vehicle position and heading
        car_x += speed * dt * (car_heading + steering_angle).cos();
        car_y += speed * dt * (car_heading + steering_angle).sin();
        car_heading += desired_yaw_rate * dt;
        println!("Car position: ({}, {}) Heading: {} Steering angle: {}", car_x, car_y, car_heading, steering_angle);
        time += dt;

        if time > 10.0 {
            break;
        }
    }

    let trace1 = Scatter::new(vec![center_x, 1.0, 0.0, -1.0, 0.0], vec![center_y, 0.0, 1.0, 0.0, -1.0])
        .name("trace1");

    let trace3 = Scatter::new(vec_x, vec_y)
        .name("trace3");

    let mut plot = Plot::new();
    plot.add_trace(trace1);
    plot.add_trace(trace3);

    let layout = Layout::new().title("<b>Line and Scatter Plot</b>".into());
    plot.set_layout(layout);

    plot.show();
}

