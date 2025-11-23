use super::*;
use crate::primitives::cube::cube;
use crate::Vec3;

#[test]
fn test_minkowski_stub() {
    let m1 = cube(Vec3::ONE, false).unwrap();
    let m2 = cube(Vec3::ONE, false).unwrap();
    // Stub relies on hull stub, which returns error
    assert!(minkowski(&m1, &m2).is_err());
}
