use crate::math::{angle_wrap, CarPosition, CarTwist, Point, Segment};

const WHEEL_SEPARATION_DISTANCE: f64 = 10.0; // cm

pub fn move_towards_point(position: CarPosition, target: Point) -> CarTwist {
    let segment = Segment(position.into(), target);
    let absolute_angle = segment.get_slope();
    let distance = segment.get_length();

    let relative_angle = angle_wrap(position.angle - absolute_angle);
    let relative_x = distance * relative_angle.sin();
    let relative_y = distance * relative_angle.cos();

    CarTwist {
        delta_x: relative_x,
        delta_y: relative_y,
        delta_theta: relative_angle,
    }
}

pub fn ackerman_forward(speed: f64, steering_angle: f64) -> CarTwist {
    let delta_x = speed * steering_angle.cos();
    let delta_y = speed * steering_angle.sin();
    let delta_theta = speed * steering_angle.tan() / WHEEL_SEPARATION_DISTANCE;

    CarTwist {
        delta_x,
        delta_y,
        delta_theta,
    }
}

pub fn ackerman_inverse_kinematics(twist: CarTwist) -> (f64, f64) {
    if twist.delta_x == 0.0 || twist.delta_y == 0.0 {
        return (0.0, 0.0);
    }

    let angle_tan = twist.delta_y / twist.delta_x;
    let speed = twist.delta_theta * WHEEL_SEPARATION_DISTANCE / angle_tan;
    let steering_angle = angle_tan.atan();

    (speed, steering_angle)
}
