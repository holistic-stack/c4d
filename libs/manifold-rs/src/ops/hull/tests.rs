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

    assert!(res.vertex_count() >= 4);
    assert!(res.face_count() >= 4);
    res.validate().expect("valid topology");
}

#[test]
fn test_hull_2d() {
    let points = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.5, 0.5, 0.0), // Inside
    ];
    let res = hull(&points).expect("hull 2d succeeds");

    assert_eq!(res.vertex_count(), 4); // Square (inner point removed)
    // 2 triangles front + 2 triangles back = 4 triangles.
    assert_eq!(res.face_count(), 4);
    res.validate().expect("valid topology");
}
