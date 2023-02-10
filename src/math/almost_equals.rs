pub trait AlmostEquals {
    fn almost_equals(self, other: Self, tolerance: Self) -> bool;
}

impl AlmostEquals for f32 {
    #[inline]
    fn almost_equals(self, other: Self, tolerance: Self) -> bool {
        (self - other).abs() < tolerance
    }
}

impl AlmostEquals for f64 {
    #[inline]
    fn almost_equals(self, other: Self, tolerance: Self) -> bool {
        (self - other).abs() < tolerance
    }
}
