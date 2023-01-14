use std::f64::consts::PI;

use crate::math::{AngleWrap, CarPosition, CarSpeed, CarTwist, Point, Segment};

// TODO measure these
const LONGITUDINAL_WHEEL_SEPARATION_DISTANCE: f64 = 26.0; // cm
const MAX_SPEED: f64 = 25.0; // cm/s
const MAX_ACCELERATION: f64 = 50.0; // cm/s^2
const MAX_STEERING_ANGLE: f64 = PI / 5.0; // around 36 deg

pub fn move_towards_point(position: &CarPosition, target: Point) -> CarTwist {
    let segment = Segment(position.into(), target);
    let absolute_angle = segment.get_slope();
    let distance = segment.get_length();

    let relative_angle = (position.angle - absolute_angle).angle_wrap();
    let relative_x = distance * relative_angle.cos();
    let relative_y = distance * relative_angle.sin();

    CarTwist {
        delta_x: relative_x,
        delta_y: relative_y,
        delta_angle: relative_angle.shortcut_wrap(),
    }
}

pub fn ackerman_forward_kinematics(car_speed: &CarSpeed) -> CarTwist {
    let CarSpeed { speed, angle } = *car_speed;

    let delta_x = speed * angle.cos();
    let delta_y = speed * angle.sin();
    let delta_theta = speed * angle.tan() / LONGITUDINAL_WHEEL_SEPARATION_DISTANCE;

    CarTwist {
        delta_x,
        delta_y,
        delta_angle: delta_theta,
    }
}

pub fn ackerman_inverse_kinematics(twist: &CarTwist) -> CarSpeed {
    if twist.delta_x == 0.0 || twist.delta_y == 0.0 {
        return CarSpeed::default();
    }

    let steering_angle_tan = twist.delta_y / twist.delta_x;
    let speed = twist.delta_angle * LONGITUDINAL_WHEEL_SEPARATION_DISTANCE / steering_angle_tan;
    let angle = steering_angle_tan.atan();

    CarSpeed {
        speed: speed.abs().min(MAX_SPEED * speed.signum()),
        angle: angle.abs().min(MAX_STEERING_ANGLE * angle.signum()),
    }
}
