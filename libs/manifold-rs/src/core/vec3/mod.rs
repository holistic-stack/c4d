//! Thin wrapper around `glam::DVec3` shared across kernel modules.
//!
//! The helper functions keep vector creation readable while avoiding direct
//! dependencies on `glam` from higher layers.

pub use glam::DVec3 as Vec3;

/// Creates a zero vector using `glam::DVec3`.
///
/// # Examples
/// ```
/// use manifold_rs::Vec3;
/// let v = manifold_rs::core::vec3::zero();
/// assert_eq!(v, Vec3::new(0.0, 0.0, 0.0));
/// ```
pub fn zero() -> Vec3 {
    Vec3::new(0.0, 0.0, 0.0)
}

/// Creates a unit length vector along the X axis.
///
/// # Examples
/// ```
/// use manifold_rs::core::vec3::unit_x;
/// let v = unit_x();
/// assert_eq!(v.x, 1.0);
/// ```
pub fn unit_x() -> Vec3 {
    Vec3::new(1.0, 0.0, 0.0)
}

#[cfg(test)]
mod tests;
