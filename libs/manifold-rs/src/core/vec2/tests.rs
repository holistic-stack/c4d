use super::*;

#[test]
fn test_vec2_creation() {
    let v = Vec2::new(1.0, 2.0);
    assert_eq!(v.x, 1.0);
    assert_eq!(v.y, 2.0);
}

#[test]
fn test_zero() {
    let v = zero();
    assert_eq!(v, Vec2::new(0.0, 0.0));
}

#[test]
fn test_units() {
    assert_eq!(unit_x(), Vec2::new(1.0, 0.0));
    assert_eq!(unit_y(), Vec2::new(0.0, 1.0));
}
