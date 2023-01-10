use std::f64::consts::PI;

pub trait AngleWrap {
    /**
     * Wrapping in interval [-PI, PI]
     */
    fn angle_wrap(self) -> Self;

    /**
     * Automatically figures out if the shortest distance between two angles on the trigonometric circle is left or right
     * and returns this newly computed angle
     */
    fn shortcut_wrap(self) -> Self;
}

impl AngleWrap for f64 {
    fn angle_wrap(self) -> f64 {
        let mut new_angle = (self + PI) % (2.0 * PI);

        if new_angle < 0.0 {
            new_angle += 2.0 * PI
        }

        new_angle - PI
    }

    fn shortcut_wrap(self) -> f64 {
        if self.abs() < (2.0 * PI + self).abs() {
            self.angle_wrap()
        } else {
            (PI * 2.0 + self).angle_wrap()
        }
    }
}
