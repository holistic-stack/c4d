use super::*;
use crate::primitives::cube::cube;

#[test]
fn test_resize_explicit() {
    let mut m = cube(Vec3::ONE, false).unwrap();
    resize(&mut m, Vec3::new(2.0, 3.0, 4.0), [false, false, false]);
    let (min, max) = m.bounding_box();
    assert_eq!(max - min, Vec3::new(2.0, 3.0, 4.0));
}

#[test]
fn test_resize_auto() {
    let mut m = cube(Vec3::ONE, false).unwrap();
    // resize([2, 0, 0], auto=[false, true, true]) -> scale X to 2 (factor 2).
    // Auto Y/Z should pick up factor 2.
    resize(&mut m, Vec3::new(2.0, 0.0, 0.0), [false, true, true]);
    let (min, max) = m.bounding_box();
    assert_eq!(max - min, Vec3::splat(2.0));
}
