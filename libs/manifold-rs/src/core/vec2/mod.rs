//! 2D vector definitions for the geometry kernel.
//!
//! Provides type alias for `glam::DVec2` and common utilities.

pub use glam::DVec2 as Vec2;

/// Creates a zero vector using `glam::DVec2`.
///
/// # Examples
/// ```
/// use manifold_rs::core::vec2::zero;
/// use manifold_rs::core::vec2::Vec2;
///
/// let v = zero();
/// assert_eq!(v, Vec2::new(0.0, 0.0));
/// ```
pub fn zero() -> Vec2 {
    Vec2::new(0.0, 0.0)
}

/// Creates a unit vector along the X axis.
pub fn unit_x() -> Vec2 {
    Vec2::new(1.0, 0.0)
}

/// Creates a unit vector along the Y axis.
pub fn unit_y() -> Vec2 {
    Vec2::new(0.0, 1.0)
}

#[cfg(test)]
mod tests;
