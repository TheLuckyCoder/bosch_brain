use crate::math::kinematics::move_towards_point;
use crate::math::{CarPosition, CarTwist, Point};
use crate::track;
use crate::track::find_path;

#[test]
fn follow_track() {
    let track = track::get_track();
    let start_node = track.get_node_by_id(24).unwrap();
    let end_node = track.get_node_by_id(60).unwrap();

    let mut path = find_path(track, start_node, end_node).unwrap();

    let mut pos = CarPosition::new(start_node.get_x() as f64, start_node.get_y() as f64, 0.0);
    println!("Path length: {}", path.len());

    loop {
        let next_node = path[0];
        let target_point = Point::new(next_node.get_x() as f64, next_node.get_y() as f64);
        let (speed, steering_angle) = move_towards_point(pos, target_point);

        pos = pos + CarTwist::new(speed, 0.0, steering_angle);

        let x = Point::from(pos).distance_to(target_point);
        if x < 2.0 {
            path.remove(0);
        }

        println!("Target {:?}", target_point);
        println!("{:?}", pos);

        if path.is_empty() {
            break;
        }
    }
}
