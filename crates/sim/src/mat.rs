//! The FLL competition mat.

/// A rectangular mat the robot drives on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat {
    /// Width (along `x`) in mm
    pub width_mm: f32,
    /// Depth (along `y`) in mm
    pub height_mm: f32,
}

impl Mat {
    /// The standard FIRST LEGO League mat (about 2362 mm × 1143 mm).
    pub const FLL: Mat = Mat {
        width_mm: 2362.0,
        height_mm: 1143.0,
    };

    /// `true` if the point lies on the mat.
    pub fn contains(&self, x_mm: f32, y_mm: f32) -> bool {
        (0.0..=self.width_mm).contains(&x_mm) && (0.0..=self.height_mm).contains(&y_mm)
    }
}

impl Default for Mat {
    fn default() -> Self {
        Mat::FLL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_points_on_the_mat() {
        let mat = Mat::default();
        assert!(mat.contains(0.0, 0.0));
        assert!(mat.contains(2362.0, 1143.0));
        assert!(!mat.contains(-1.0, 10.0));
        assert!(!mat.contains(10.0, 1200.0));
    }
}
