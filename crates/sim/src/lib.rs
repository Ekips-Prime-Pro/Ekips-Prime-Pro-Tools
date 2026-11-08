//! 2D simulator for Ekips Prime Pro robots on an FLL mat.
//!
//! **Status: skeleton.** The simulator will run programs as soon as the
//! Core repository has a VM. Until then this crate only defines the world
//! model and the robot geometry. The full plan is in
//! `docs/en/simulator.md` (German: `docs/de/simulator.md`).
//!
//! Conventions (the same as in `ekips-hub-api`):
//! - Lengths in millimetres, angles in degrees, time in milliseconds.
//! - The mat's origin is its bottom-left corner; `x` grows to the right,
//!   `y` grows away from the student (up on screen).
//! - A heading of `0°` points along `+y`; positive headings turn the robot
//!   **clockwise** seen from above, like `hub.heading` on the real hub.
//!
//! The crate has no platform-specific dependencies so that it compiles for
//! Windows, Linux, macOS and `wasm32-unknown-unknown`.

pub mod mat;
pub mod robot;

pub use mat::Mat;
pub use robot::{DriveGeometry, Pose};

/// Fixed simulation step in milliseconds. A fixed step keeps runs
/// reproducible: the same program always gives the same result.
pub const STEP_MS: u32 = 10;
