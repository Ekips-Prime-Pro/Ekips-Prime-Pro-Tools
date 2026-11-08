//! Position and movement of a two-wheeled robot.

use ekips_hub_api::DriveBaseConfig;

/// Where the robot is: the point in the middle between the two wheels.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pose {
    /// Position along the mat's width in mm
    pub x_mm: f32,
    /// Position along the mat's depth in mm
    pub y_mm: f32,
    /// Direction in degrees, `0` = `+y`, clockwise positive
    pub heading_deg: f32,
}

/// Wheel geometry needed for the movement model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DriveGeometry {
    /// Wheel diameter in mm
    pub wheel_diameter_mm: f32,
    /// Distance between the wheel contact points in mm
    pub axle_track_mm: f32,
}

impl From<DriveBaseConfig> for DriveGeometry {
    fn from(c: DriveBaseConfig) -> Self {
        DriveGeometry {
            wheel_diameter_mm: c.wheel_diameter_mm,
            axle_track_mm: c.axle_track_mm,
        }
    }
}

impl DriveGeometry {
    /// Distance a wheel rolls when its motor turns by `degrees`.
    pub fn wheel_travel_mm(&self, degrees: f32) -> f32 {
        degrees / 360.0 * core::f32::consts::PI * self.wheel_diameter_mm
    }
}

impl Pose {
    /// New pose after the left and right wheels rolled the given distances
    /// (differential drive, exact for a constant ratio during the step).
    pub fn advance(self, left_mm: f32, right_mm: f32, geometry: &DriveGeometry) -> Pose {
        let distance = (left_mm + right_mm) / 2.0;
        // Left wheel further than right wheel = clockwise = positive.
        let turn_rad = (left_mm - right_mm) / geometry.axle_track_mm;
        let start = self.heading_deg.to_radians();
        let (dx, dy) = if turn_rad.abs() < 1e-6 {
            (distance * start.sin(), distance * start.cos())
        } else {
            // Drive along a circle arc with radius `distance / turn_rad`.
            let radius = distance / turn_rad;
            let end = start + turn_rad;
            (
                radius * (start.cos() - end.cos()),
                radius * (end.sin() - start.sin()),
            )
        };
        Pose {
            x_mm: self.x_mm + dx,
            y_mm: self.y_mm + dy,
            heading_deg: self.heading_deg + turn_rad.to_degrees(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GEOMETRY: DriveGeometry = DriveGeometry {
        wheel_diameter_mm: 56.0,
        axle_track_mm: 112.0,
    };

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn straight_ahead_moves_along_heading() {
        let p = Pose::default().advance(100.0, 100.0, &GEOMETRY);
        assert!(close(p.x_mm, 0.0) && close(p.y_mm, 100.0) && close(p.heading_deg, 0.0));

        let east = Pose {
            heading_deg: 90.0,
            ..Pose::default()
        };
        let p = east.advance(50.0, 50.0, &GEOMETRY);
        assert!(close(p.x_mm, 50.0) && close(p.y_mm, 0.0), "{p:?}");
    }

    #[test]
    fn turning_on_the_spot_is_clockwise_positive() {
        // A quarter of the circle with the axle track as diameter.
        let quarter = core::f32::consts::PI * GEOMETRY.axle_track_mm / 4.0;
        let p = Pose::default().advance(quarter, -quarter, &GEOMETRY);
        assert!(close(p.heading_deg, 90.0), "{p:?}");
        assert!(close(p.x_mm, 0.0) && close(p.y_mm, 0.0), "{p:?}");
    }

    #[test]
    fn wheel_travel() {
        let one_turn = GEOMETRY.wheel_travel_mm(360.0);
        assert!(close(one_turn, core::f32::consts::PI * 56.0));
    }

    #[test]
    fn many_small_steps_match_one_big_step() {
        let mut small = Pose::default();
        for _ in 0..100 {
            small = small.advance(2.0, 1.0, &GEOMETRY);
        }
        let big = Pose::default().advance(200.0, 100.0, &GEOMETRY);
        assert!(close(small.x_mm, big.x_mm) && close(small.y_mm, big.y_mm));
    }
}
