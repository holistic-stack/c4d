use super::*;
use crate::Vec3;

#[test]
fn test_hull_box() {
    let points = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(1.0, 1.0, 1.0), // Extra point
    ];
    let res = hull(&points).expect("hull succeeds");

    // Hull of tetrahedron + 1 point?
    // Should produce a valid manifold.
    assert!(res.vertex_count() >= 4);
    assert!(res.face_count() >= 4);
    res.validate().expect("valid topology");
}
