//! Sphere primitive tests.

use super::*;

#[test]
fn test_sphere_creation() {
    let sphere = Sphere::new(10.0, 16);
    let manifold = sphere.to_manifold();
    assert!(manifold.validate().is_ok());
}

#[test]
fn test_sphere_bounding_box() {
    let sphere = Sphere::new(5.0, 16);
    let manifold = sphere.to_manifold();
    assert!(manifold.validate().is_ok());

    // Basic bounds check
    let (min, max) = manifold.bounding_box();
    // Radius 5.0, so extent should be ~5.0
    assert!(max.x > 0.0);
    assert!(min.x < 0.0);
}

#[test]
fn test_sphere_low_segments() {
    let sphere = Sphere::new(1.0, 3);
    let manifold = sphere.to_manifold();
    assert!(manifold.validate().is_ok());
}
