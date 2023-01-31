use std::f64::consts::PI;
use std::time::{Duration, Instant};

use crate::math::kinematics::{ackerman_forward_kinematics};
use crate::math::{AngleWrap, Car, CarPosition, Point, Segment};
use crate::track;
use crate::track::find_path;

#[test]
fn follow_track() {
    let track = track::get_track();
    let start_node = track.get_node_by_id(45).unwrap();
    let end_node = track.get_node_by_id(50).unwrap();
    println!("Start");
    let mut path = find_path(track, start_node, end_node).unwrap();
    path.remove(0);

    println!("Path length: {}", path.len());

    // let mut last_timestamp = Instant::now();

    let mut car = Car::new(start_node.get_x() as f64, start_node.get_y() as f64, PI, 5.0, 0.0);

    let mut next_node = path.remove(0);

    let mut target_point = Point::new(next_node.get_x() as f64, next_node.get_y() as f64);

    println!("Target {:?} (Node: {})", target_point, next_node.id);

    loop {
        print!("\n{:?}", car.position);
        println!("degrees: {}", car.position.heading_angle.to_degrees());

        let segment = Segment(Point::from(&car.position), target_point);

        let heading_angle = segment.get_angle();
        println!("headingAngle: {}", heading_angle.to_degrees());

        let relative_position = ackerman_forward_kinematics(car.speed, heading_angle, car.position.heading_angle);
        print!("rel pos: {:?}", relative_position);
        println!("degrees: {}", relative_position.heading_angle.to_degrees());

        car.position.x += relative_position.x;
        car.position.y += relative_position.y;
        car.position.heading_angle += relative_position.heading_angle;


        car.position.heading_angle = car.position.heading_angle.angle_wrap();

        // let delta_time = last_timestamp.elapsed().as_secs_f64();
        let dist = Point::from(&car.position).distance_to(target_point);
        println!("{:?}", dist);

        if dist < car.speed {
            if path.is_empty() {
                break;
            }

            next_node = path.remove(0);

            target_point = Point::new(next_node.get_x() as f64, next_node.get_y() as f64);

            println!("Target {:?} (Node: {})", target_point, next_node.id);
        }

        // last_timestamp = Instant::now();
        std::thread::sleep(Duration::from_millis(100));
    }
}
