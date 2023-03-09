use plotly::{Layout, Plot, Scatter};
use std::f64::consts::PI;
use std::time::Instant;
use plotly::common::Mode;


use crate::math::{angle_between_vectors, AngleWrap, Car, Circle, get_heading_vector_and_angle, Point, Segment};

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

    let current_position = Point::new(0.0, 1.0);

    let (mut heading_vector, mut heading_angle) = get_heading_vector_and_angle(circle, current_position,1.0);

    // let mut car = Car::new(0_f64, 1_f64 as f64, (PI).angle_wrap(), 0.1_f64, 0 as f64);
    let mut car = Car::new(current_position.x, current_position.y, heading_angle.angle_wrap(), 1.0, 0 as f64);

    let mut vec_x = Vec::new();
    let mut vec_y = Vec::new();

    let mut heading_matrix = Vec::new();

    heading_matrix.push(vec![Point::from(&car.position), heading_vector]);

    let mut c = 0;

    let distance_to_circle = Point::from(&car.position).distance_to(circle.center);
    println!("Distance to circle: {}", distance_to_circle);

    let mut last_timestamp = Instant::now();

    loop {
        c += 1;
        if c as f64 > 5000.0 {
            break;
        }

        vec_x.push(car.position.x);
        vec_y.push(car.position.y);

        // let delta_time = last_timestamp.elapsed().as_secs_f64();
        let delta_time = 0.1;

        car.steering_angle = LONGITUDINAL_WHEEL_SEPARATION_DISTANCE.atan2(distance_to_circle);

        // let change_of_heading_angle = car.speed * delta_time * car.steering_angle.tan() / LONGITUDINAL_WHEEL_SEPARATION_DISTANCE;
        let change_of_heading_angle = car.speed * delta_time / distance_to_circle;

        car.position.x += car.speed * delta_time * car.position.heading_angle.cos();
        car.position.y += car.speed * delta_time * car.position.heading_angle.sin();

        car.position.heading_angle = (car.position.heading_angle + change_of_heading_angle).angle_wrap();

        println!("Steering Angle: {:.3}", car.steering_angle.to_degrees());
        println!("Heading Angle: {:.3}", car.position.heading_angle.to_degrees());

        (heading_vector,heading_angle) = get_heading_vector_and_angle(circle, Point::from(&car.position), 1.0);

        heading_matrix.push(vec![Point::from(&car.position), heading_vector]);

        last_timestamp = Instant::now();
        // std::thread::sleep(std::time::Duration::from_millis(100));
        println!()
    }

    let mut plot = Plot::new();

    let path = Scatter::new(vec_x, vec_y)
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


    for (c, vector) in heading_matrix.into_iter().enumerate() {
        if c % 5 == 0 {
            let current_pos = vector[0];
            let heading_vector = vector[1];

            let trace_heading = Scatter::new(vec![current_pos.x, heading_vector.x], vec![current_pos.y, heading_vector.y])
                .name("Heading".to_string() + &c.to_string())
                .mode(Mode::LinesMarkers);

            plot.add_trace(trace_heading);
        }
    }


    plot.add_trace(trace_circle);
    plot.add_trace(path);

    let layout = Layout::new().title("<b>Steering around the unit circle</b>".into());
    plot.set_layout(layout);

    plot.show();
}
