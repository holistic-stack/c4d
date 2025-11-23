use super::*;
use crate::primitives::cube::cube;
use crate::Vec3;

#[test]
fn test_minkowski_convex() {
    let m1 = cube(Vec3::ONE, false).unwrap();
    let m2 = cube(Vec3::ONE, false).unwrap();

    // Minkowski sum of two unit cubes (1x1x1 at origin corner)
    // Result should be a 2x2x2 cube (0 to 2).
    let res = minkowski(&m1, &m2).expect("minkowski succeeds");

    let (min, max) = res.bounding_box();
    assert_eq!(min, Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(max, Vec3::new(2.0, 2.0, 2.0));

    res.validate().expect("valid topology");
}
