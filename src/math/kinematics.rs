use std::f64::consts::PI;

use crate::math::{AngleWrap, CarPosition, Point, Segment, Car};

// TODO measure these
const LONGITUDINAL_WHEEL_SEPARATION_DISTANCE: f64 = 26.0;
// cm
const MAX_SPEED: f64 = 25.0;
// cm/s
const MAX_ACCELERATION: f64 = 50.0;
// cm/s^2
const MAX_STEERING_ANGLE: f64 = 0.436332; // around 25 deg

// pub fn move_towards_point(position: &CarPosition, target: Point) -> CarPos {
//     let segment = Segment(position.into(), target);
//     let absolute_angle = segment.get_slope();
//     let distance = segment.get_length();
//
//     let relative_angle = (position.angle - absolute_angle).angle_wrap();
//     let relative_x = distance * relative_angle.cos();
//     let relative_y = distance * relative_angle.sin();
//
//     CarPos {
//         delta_x: relative_x,
//         delta_y: relative_y,
//         delta_angle: relative_angle,
//     }
// }

pub fn ackerman_forward_kinematics(speed: f64, heading_angle: f64, previous_angle: f64) -> CarPosition {
    let change_of_x_rel_to_current_pos = speed * heading_angle.cos();
    let change_of_y_rel_to_current_pos = speed * heading_angle.sin();
    let theta_deriv = (previous_angle - heading_angle).abs();
    let mut steering_angle = (LONGITUDINAL_WHEEL_SEPARATION_DISTANCE * theta_deriv / speed).atan();

    if steering_angle > MAX_STEERING_ANGLE {
        steering_angle = MAX_STEERING_ANGLE;
    }

    CarPosition {
        x: change_of_x_rel_to_current_pos,
        y: change_of_y_rel_to_current_pos,
        angle: steering_angle,
    }
}

