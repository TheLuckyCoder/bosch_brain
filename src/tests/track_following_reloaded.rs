use std::env;
use std::f64::consts::PI;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};
use plotly::common::Mode;
use plotly::{Layout, Plot, Scatter};


use crate::math::{AngleWrap, Car, Circle, Point, Segment};
use crate::track;
use crate::track::find_path;


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

#[test]
fn follow_track() {
    let track = track::get_track();
    let start_node = track.get_node_by_id(46).unwrap();
    let end_node = track.get_node_by_id(50).unwrap();

    println!("Start");

    let mut path = find_path(track, start_node, end_node).unwrap();

    let mut path_vec_x = Vec::new();
    let mut path_vec_y = Vec::new();

    for node in &path {
        path_vec_x.push(node.get_x() as f64);
        path_vec_y.push(node.get_y() as f64);
    }

    path.remove(0);

    println!("Path length: {}", path.len());

    let mut next_two_nodes = vec![path.remove(0), path.remove(0)];

    println!("NextNodes: {:?}", next_two_nodes);

    let mut car = Car::new(start_node.get_x() as f64, start_node.get_y() as f64, PI, 1.0, 0.0);

    println!("Target {} (Node: {})", Point::from(next_two_nodes[0]), next_two_nodes[0].id);

    const LONGITUDINAL_WHEEL_SEPARATION_DISTANCE: f64 = 26.0; // cm

    let mut vec_x = Vec::new();

    let mut vec_y = Vec::new();

    loop {
        vec_x.push(car.position.x);
        vec_y.push(car.position.y);

        println!("\n {}", car);

        let dist_from_car_to_next_node = Point::from(&car.position).distance_to(next_two_nodes[0].into());
        println!("\nDist To next Node: {:.3}", dist_from_car_to_next_node);

        if dist_from_car_to_next_node > 40.0 {
            break;
        }

        if dist_from_car_to_next_node < car.speed {
            if path.is_empty() {
                break;
            }

            next_two_nodes.remove(0);

            next_two_nodes.push(path.remove(0));

            println!("NextNodes: {:?}", next_two_nodes);

            println!("\nTarget {} (Node: {})", Point::from(next_two_nodes[0]), next_two_nodes[0].id);
        }

        let circle = Circle::find_center(Point::from(&car.position),next_two_nodes[0].into(), next_two_nodes[1].into());

        println!("Circle: {:?}", circle);

        let distance_to_circle = Point::from(&car.position).distance_to(circle.center);

        println!("Distance to circle: {:.3}", distance_to_circle);

        // car.steering_angle = (LONGITUDINAL_WHEEL_SEPARATION_DISTANCE / distance_to_circle).atan();

        let change_of_heading_angle = car.speed / distance_to_circle;

        // let change_of_heading_angle_2 = car.speed * car.steering_angle.tan() / LONGITUDINAL_WHEEL_SEPARATION_DISTANCE;

        // println!("Change of Heading Angle: {:.3} {:.3}", change_of_heading_angle, change_of_heading_angle_2);

        car.position.heading_angle += change_of_heading_angle;

        car.position.heading_angle = car.position.heading_angle.angle_wrap();

        let change_of_x = car.speed * car.position.heading_angle.cos();

        let change_of_y = car.speed * car.position.heading_angle.sin();

        car.position.x += change_of_x;
        car.position.y += change_of_y;

        // car.position.heading_angle = car.position.heading_angle.angle_wrap();

        // last_timestamp = Instant::now();
        std::thread::sleep(Duration::from_millis(100));

        stdout().flush().unwrap();
    }

    let trace1 = Scatter::new(path_vec_x, path_vec_y)
        .name("trace1");

    let trace3 = Scatter::new(vec_x,vec_y)
        .name("trace3");

    let mut plot = Plot::new();
    plot.add_trace(trace1);
    plot.add_trace(trace3);

    let layout = Layout::new().title("<b>Line and Scatter Plot</b>".into());
    plot.set_layout(layout);

    plot.show();
}
