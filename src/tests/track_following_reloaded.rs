use plotly::common::{Line, Mode};
use plotly::{Layout, Plot, Scatter};

use std::f64::consts::PI;

use std::time::{Instant};
use plotly::layout::Axis;


use crate::math::{AngleWrap, Car, Circle, Point};
use crate::track;
use crate::track::{find_path, TrackNode};

fn line_and_scatter_plot() {
    let trace1 = Scatter::new(vec![1, 2, 3, 4], vec![10, 15, 13, 17])
        .name("trace1")
        .mode(Mode::Markers);
    let trace2 = Scatter::new(vec![2, 3, 4, 5], vec![16, 5, 11, 9])
        .name("trace2")
        .mode(Mode::Lines);
    let trace3 = Scatter::new(vec![1, 2, 3, 4], vec![12, 9, 15, 12])
        .name("trace3");

    let mut plot = Plot::new();
    plot.add_trace(trace1);
    plot.add_trace(trace2);
    plot.add_trace(trace3);

    let layout = Layout::new().title("<b>Line and Scatter Plot</b>".into());
    plot.set_layout(layout);

    plot.show();
}

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
fn follow_track() {
    let track = track::get_track();
    let start_node = track.get_node_by_id(46).unwrap();
    let end_node = track.get_node_by_id(59).unwrap();

    let mut path = find_path(track, start_node, end_node).unwrap();

    let car = Car::new(
        start_node.get_x() as f64,
        start_node.get_y() as f64,
        (PI / 1_f64).angle_wrap(),
        4.0,
        0.0,
    );

    follow_path(car, path, 50.0, 10.0);
}

#[test]
fn follow_circle() {
    let start_node = &TrackNode::new(0, 0_f32, 10_f32, Vec::new());

    let start_node = &TrackNode::new(0, 1.96_f32, 9.81_f32, Vec::new());

    let left_node = &TrackNode::new(2, -10_f32, 0_f32, Vec::new());

    let bottom_node = &TrackNode::new(3, 0_f32, -10_f32, Vec::new());

    let next_node = &TrackNode::new(4, 10_f32, 0_f32, Vec::new());

    let end_node = &TrackNode::new(1, 0_f32, 10_f32, Vec::new());

    let mut path = Vec::new();

    path.push(start_node);
    path.push(left_node);
    path.push(bottom_node);
    path.push(next_node);
    path.push(end_node);

    let car = Car::new(
        start_node.get_x() as f64,
        start_node.get_y() as f64,
        (PI / 1_f64).angle_wrap(),
        1.0,
        0.0,
    );

    follow_path(car, path, 20.0, 0.1);
}

#[test]
fn follow_test() {
    let track = track::get_track();
    let start_node = track.get_node_by_id(40).unwrap();
    let end_node = track.get_node_by_id(73).unwrap();

    let mut path = find_path(track, start_node, end_node).unwrap();

    let car = Car::new(
        start_node.get_x() as f64,
        start_node.get_y() as f64,
        (PI / 1_f64).angle_wrap(),
        4.0,
        0.0,
    );

    follow_path(car, path, 100.0, 10.0);
}


fn follow_path(mut car: Car, mut path: Vec<&TrackNode>, max_allowed_distance: f32, min_dist_to_next_waypoint: f32) {
    let mut path_vec_x = Vec::new();
    let mut path_vec_y = Vec::new();

    for node in &path {
        path_vec_x.push(node.get_x() as f64);
        path_vec_y.push(node.get_y() as f64);
    }

    println!("Car: {:?}", car);

    path.remove(0);

    println!("Path length: {}", path.len());

    let mut next_two_nodes = vec![path.remove(0), path.remove(0)];

    println!("NextNodes: {:?}", next_two_nodes);

    println!(
        "Target {} (Node: {})",
        Point::from(next_two_nodes[0]),
        next_two_nodes[0].id
    );

    const LONGITUDINAL_WHEEL_SEPARATION_DISTANCE: f64 = 26.0; // cm

    let mut matrix = Vec::new();

    let mut vec_x = Vec::new();

    let mut vec_y = Vec::new();

    let mut circles = Vec::new();

    let mut last_timestamp = Instant::now();

    let circle = Circle::find_center(
        Point::from(&car.position),
        next_two_nodes[0].into(),
        next_two_nodes[1].into(),
    );

    let distance_to_circle = Point::from(&car.position).distance_to(circle.center);
    println!("Circle: {:?}", circle);
    println!("Distance to circle: {:.3}", distance_to_circle);

    circles.push(points_on_circle(circle, 50));

    loop {
        println!("current pos: {:?} ", car.position);
        vec_x.push(car.position.x);
        vec_y.push(car.position.y);

        let dist_from_car_to_next_node = Point::from(&car.position).distance_to(next_two_nodes[0].into());

        // println!("\nDist To next Node: {:.3}", dist_from_car_to_next_node);

        if dist_from_car_to_next_node > max_allowed_distance as f64 {
            matrix.push((vec_x.clone(), vec_y.clone()));

            vec_x.clear();
            vec_y.clear();

            println!("Too far away from next node");
            break;
        }

        if dist_from_car_to_next_node < min_dist_to_next_waypoint as f64 {
            matrix.push((vec_x.clone(), vec_y.clone()));

            vec_x.clear();
            vec_y.clear();

            if path.is_empty() && next_two_nodes.len() == 1 {
                println!("Reached end of path");
                break;
            }

            next_two_nodes.remove(0);

            if !path.is_empty() {
                next_two_nodes.push(path.remove(0));

                let circle = Circle::find_center(
                    Point::from(&car.position),
                    next_two_nodes[0].into(),
                    next_two_nodes[1].into(),
                );

                let distance_to_circle = Point::from(&car.position).distance_to(circle.center);
                println!("Circle: {:?}", circle);
                println!("Distance to circle: {:.3}", distance_to_circle);

                circles.push(points_on_circle(circle, 50));
            }
            println!("NextNodes: {:?}", next_two_nodes);
            println!("\nTarget {} (Node: {})", Point::from(next_two_nodes[0]), next_two_nodes[0].id);
        }
        // let delta_time = last_timestamp.elapsed().as_secs_f64();
        let delta_time = 0.1;

        car.steering_angle = LONGITUDINAL_WHEEL_SEPARATION_DISTANCE.atan2(distance_to_circle);
        let change_of_heading_angle = car.speed * delta_time / distance_to_circle;

        // println!("\nSimple Sterrering Angle: {:.3}", car.steering_angle);
        // println!("Simple Change of Heading Angle: {:.3}", change_of_heading_angle);

        // car.steering_angle = (LONGITUDINAL_WHEEL_SEPARATION_DISTANCE / distance_to_circle).atan2(1.0);
        // let change_of_heading_angle =
        //     car.speed * delta_time * car.steering_angle.tan() / LONGITUDINAL_WHEEL_SEPARATION_DISTANCE;
        //
        // println!("Sterrering Angle: {:.3}", car.steering_angle);
        // println!("Change of Heading Angle: {:.3}", change_of_heading_angle);

        car.position.x += car.speed * delta_time * car.position.heading_angle.cos();
        car.position.y += car.speed * delta_time * car.position.heading_angle.sin();

        car.position.heading_angle = (car.position.heading_angle + change_of_heading_angle).angle_wrap();

        last_timestamp = Instant::now();
        // std::thread::sleep(Duration::from_millis(1000));
    }

    let mut plot = Plot::new();

    let mut layout = Layout::new()
        .title("<b>Circle pathing</b>".into());

    let trace1 = Scatter::new(path_vec_x, path_vec_y)
        .name("Path");
    // .line(Line::new().color("red"));
    plot.add_trace(trace1);

    for (c, (xs, ys)) in matrix.into_iter().enumerate() {
        let trace_circle = Scatter::new(xs, ys)
            .name("Points".to_string() + &c.to_string())
            .mode(Mode::Markers);

        plot.add_trace(trace_circle);
    }

    for (c, circle) in circles.into_iter().enumerate() {
        if c % 1 == 0 {
            let mut vec_x = Vec::new();
            let mut vec_y = Vec::new();
            for point in circle {
                vec_x.push(point.x);
                vec_y.push(point.y);
            }
            vec_x.push(vec_x[0]);
            vec_y.push(vec_y[0]);

            let trace_circle = Scatter::new(vec_x, vec_y)
                .name("circle".to_string() + &c.to_string())
                .mode(Mode::Lines);

            plot.add_trace(trace_circle);
        }
    }

    plot.set_layout(layout);

    plot.show();
}
