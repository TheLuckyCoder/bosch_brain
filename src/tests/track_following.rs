use std::time::{Duration, Instant};

use crate::math::kinematics::move_towards_point;
use crate::math::{AngleWrap, CarPosition, Point};
use crate::track;
use crate::track::find_path;

#[test]
fn follow_track() {
    let track = track::get_track();
    let start_node = track.get_node_by_id(39).unwrap();
    let end_node = track.get_node_by_id(44).unwrap();

    let mut path = find_path(track, start_node, end_node).unwrap();
    path.remove(0);

    let mut pos = CarPosition::new(start_node.get_x() as f64, start_node.get_y() as f64, 0.0);
    println!("Path length: {}", path.len());

    let mut last_timestamp = Instant::now();

    while !path.is_empty() {
        let next_node = path[0];
        let target_point = Point::new(next_node.get_x() as f64, next_node.get_y() as f64);

        let twist = move_towards_point(&pos, target_point);
        let delta_time = last_timestamp.elapsed().as_secs_f64();

        let diluted_car_twist = twist.rotate(pos.angle) * delta_time;
        pos = pos + &diluted_car_twist;
        pos.angle = pos.angle.angle_wrap();

        let x = Point::from(&pos).distance_to(target_point);
        if x < 1.0 {
            path.remove(0);
        }

        println!("Target {:?} (Node: {})", target_point, next_node.id);
        println!("{:?}", twist);
        println!("{:?}\n", pos);

        last_timestamp = Instant::now();
        std::thread::sleep(Duration::from_millis(200));
    }
}
