use super::*;
use crate::Vec3;

#[test]
fn test_hull_stub() {
    let points = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    ];
    // Currently stub returns error
    assert!(hull(&points).is_err());
}
