use super::*;

#[test]
fn test_circle_validity() {
    let m = circle(10.0, 20).unwrap();
    assert!(m.validate().is_ok());
    // With earcut triangulation of a circle, we just have the contour points.
    // Earcut doesn't necessarily add a center point (Steiner point) unless we force it.
    // So vertex count should be 20.
    assert_eq!(m.vertex_count(), 20);
    // Faces: (20 - 2) triangles * 2 sides = 18 * 2 = 36 faces.
    assert_eq!(m.face_count(), 36);
}

#[test]
fn test_circle_bounding_box() {
    let m = circle(10.0, 360).unwrap(); // High segments for close approximation
    let (min, max) = m.bounding_box();

    assert!((min.x - -10.0).abs() < 0.1);
    assert!((min.y - -10.0).abs() < 0.1);
    assert!((max.x - 10.0).abs() < 0.1);
    assert!((max.y - 10.0).abs() < 0.1);
}

#[test]
fn test_invalid_circle() {
    assert!(circle(-1.0, 10).is_err());
    assert!(circle(1.0, 2).is_err());
}
