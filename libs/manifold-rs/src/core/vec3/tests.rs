//! Tests for the Vec3 helpers.

use super::*;

/// Ensures zero helper returns the expected vector.
///
/// # Examples
/// ```
/// use manifold_rs::core::vec3;
/// assert_eq!(vec3::zero(), vec3::Vec3::new(0.0, 0.0, 0.0));
/// ```
#[test]
fn zero_returns_origin() {
    assert_eq!(zero(), Vec3::new(0.0, 0.0, 0.0));
}

/// Ensures unit_x helper yields normalized vector.
///
/// # Examples
/// ```
/// use manifold_rs::core::vec3;
/// let v = vec3::unit_x();
/// assert_eq!(v.length(), 1.0);
/// ```
#[test]
fn unit_x_is_normalized() {
    let v = unit_x();
    assert!((v.length() - 1.0).abs() < f64::EPSILON);
}
