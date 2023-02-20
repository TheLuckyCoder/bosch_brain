use plotly::{Layout, Plot, Scatter};
use std::f64::consts::PI;
use std::time::Instant;
use plotly::common::Mode;


use crate::math::{AngleWrap, Car, Circle, Point, Segment};

const MAX_STEERING_ANGLE: f64 = 0.4363323129985824; // 25 degrees

const LONGITUDINAL_WHEEL_SEPARATION_DISTANCE: f64 = 1.0; // cm

fn points_on_circle(circle: Circle, nr_of_points: i32) -> Vec<Point> {
    let mut result = Vec::new();

    let angle = 2.0 * PI / nr_of_points as f64;
    for i in 0..nr_of_points {
        let x = circle.center.x + circle.radius * (angle * i as f64).cos();
        let y = circle.center.y + circle.radius * (angle * i as f64).sin();
        result.push(Point::new(x, y));
    }
    result
}

#[test]
fn follow_circle() {
    let circle = Circle::find_center(
        Point::new(1.0, 0.0),
        Point::new(0.0, 1.0),
        Point::new(-1.0, 0.0),
    );

    println!("Circle: {:?}", circle);

    // let mut car = Car::new(0_f64, 1_f64 as f64, (PI).angle_wrap(), 0.1_f64, 0 as f64);
    let mut car = Car::new(0_f64, 1_f64, (PI / 1_f64).angle_wrap(), 1.0/30.0, 0 as f64);

    let mut vec_x = Vec::new();

    let mut vec_y = Vec::new();

    let mut c = 0;

    let mut last_timestamp = Instant::now();

    let distance_to_circle = Point::from(&car.position).distance_to(circle.center);
    print!("Distance to circle: {}", distance_to_circle);

    loop {
        c += 1;
        if c as f64 > 5000.0 {
            break;
        }

        vec_x.push(car.position.x);
        vec_y.push(car.position.y);

        // let delta_time = last_timestamp.elapsed().as_secs_f64();
        let delta_time = 0.1;

        if false {
            // let desired_yaw_rate = car.speed / distance_to_circle;
            //
            // car.steering_angle = desired_yaw_rate * LONGITUDINAL_WHEEL_SEPARATION_DISTANCE / car.speed;
            //
            // car.position.x += car.speed * delta_time * (car.position.heading_angle + car.steering_angle).cos();
            // car.position.y += car.speed * delta_time * (car.position.heading_angle + car.steering_angle).sin();
            // car.position.heading_angle += desired_yaw_rate * delta_time
        } else {
            car.steering_angle = LONGITUDINAL_WHEEL_SEPARATION_DISTANCE.atan2(distance_to_circle);

            // let change_of_heading_angle = car.speed * delta_time * car.steering_angle.tan() / LONGITUDINAL_WHEEL_SEPARATION_DISTANCE;
            let change_of_heading_angle = car.speed * delta_time / distance_to_circle;

            car.position.x += car.speed * delta_time * car.position.heading_angle.cos();
            car.position.y += car.speed * delta_time * car.position.heading_angle.sin();

            car.position.heading_angle = (car.position.heading_angle + change_of_heading_angle).angle_wrap();
        }
        println!("Steering Angle: {:.3}", car.steering_angle.to_degrees());
        println!("Heading Angle: {:.3}", car.position.heading_angle.to_degrees());

        last_timestamp = Instant::now();
        // std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let trace3 = Scatter::new(vec_x, vec_y)
        .name("path")
        .mode(Mode::LinesMarkers);

    let circle_points = points_on_circle(circle, 1000);

    let mut vec_x = Vec::new();
    let mut vec_y = Vec::new();
    for point in circle_points.iter() {
        vec_x.push(point.x);
        vec_y.push(point.y);
    }
    vec_x.push(vec_x[0]);
    vec_y.push(vec_y[0]);

    let trace_circle = Scatter::new(vec_x, vec_y)
        .name("circle")
        .mode(Mode::Lines);


    let mut plot = Plot::new();
    plot.add_trace(trace_circle);
    plot.add_trace(trace3);

    let layout = Layout::new().title("<b>Steering around the unit circle</b>".into());
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

    let radius = ((circle_center.x - car.position.x).powi(2)
        + (circle_center.y - car.position.y).powi(2))
        .sqrt();

    let mut error_heading_derivative = 0.0;
    let mut time = 0.0;
    let dt = 0.1;
    let kd_heading = 0.1;
    let kp_heading = 0.1;

    loop {
        // calculate heading error
        let error_heading =
            car.position.heading_angle - (car.position.heading_angle - (car.speed / radius));
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
        println!(
            "time: {:.1}s, x: {:.2}, y: {:.2}, heading: {:.2}, steering_angle: {:.2}",
            time, car.position.x, car.position.x, car.position.heading_angle, steering_angle
        );

        // stop the simulation after a certain amount of time
        if time > 10.0 {
            break;
        }
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
}

#[test]
fn test_lateral_control() {
    let circle_center = Point::new(0.0, 0.0);

    let car = Car::new(10_f64, 0 as f64, PI / 2_f64, 2_f64, 0 as f64);

    let wheelbase = 2.0;

    lateral_control(
        circle_center.x,
        circle_center.y,
        car.position.x,
        car.position.y,
        car.position.heading_angle,
        car.speed,
        wheelbase,
    );
}

fn lateral_control(
    center_x: f64,
    center_y: f64,
    car_x: f64,
    car_y: f64,
    car_heading: f64,
    speed: f64,
    wheelbase: f64,
) {
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
        println!(
            "Car position: ({}, {}) Heading: {} Steering angle: {}",
            car_x, car_y, car_heading, steering_angle
        );
        time += dt;

        if time > 10.0 {
            break;
        }
    }

    let trace1 = Scatter::new(
        vec![center_x, 1.0, 0.0, -1.0, 0.0],
        vec![center_y, 0.0, 1.0, 0.0, -1.0],
    )
        .name("trace1");

    let trace3 = Scatter::new(vec_x, vec_y).name("trace3");

    let mut plot = Plot::new();
    plot.add_trace(trace1);
    plot.add_trace(trace3);

    let layout = Layout::new().title("<b>Line and Scatter Plot</b>".into());
    plot.set_layout(layout);

    plot.show();
}
