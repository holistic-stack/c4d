use super::*;

#[test]
fn test_square_validity() {
    let m = square(DVec2::new(10.0, 10.0), false).unwrap();
    assert!(m.validate().is_ok());
    assert_eq!(m.vertex_count(), 4);
    assert_eq!(m.face_count(), 4); // Double sided = 2 * 2 triangles
}

#[test]
fn test_square_centered() {
    let m = square(DVec2::new(10.0, 10.0), true).unwrap();
    let (min, max) = m.bounding_box();

    assert!((min.x - -5.0).abs() < 1e-6);
    assert!((min.y - -5.0).abs() < 1e-6);
    assert!((max.x - 5.0).abs() < 1e-6);
    assert!((max.y - 5.0).abs() < 1e-6);
}

#[test]
fn test_square_not_centered() {
    let m = square(DVec2::new(10.0, 10.0), false).unwrap();
    let (min, max) = m.bounding_box();

    assert!((min.x - 0.0).abs() < 1e-6);
    assert!((min.y - 0.0).abs() < 1e-6);
    assert!((max.x - 10.0).abs() < 1e-6);
    assert!((max.y - 10.0).abs() < 1e-6);
}

#[test]
fn test_invalid_size() {
    let res = square(DVec2::new(-10.0, 10.0), false);
    assert!(res.is_err());
}
