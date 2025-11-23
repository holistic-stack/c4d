use super::*;
use crate::Vec3;

#[test]
fn test_polyhedron_tetrahedron() {
    let points = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
    ];
    // Tetrahedron faces (CCW from outside)
    let faces = vec![
        vec![0, 2, 1], // Bottom (Z=0, looking down -> CW from top -> CCW from bottom?) 0->2->1 is CW from top. Normal -Z. Correct.
        vec![0, 1, 3], // Front (Y=0)
        vec![1, 2, 3], // Hypotenuse face
        vec![2, 0, 3], // Left (X=0)
    ];

    let m = polyhedron(&points, &faces).expect("polyhedron succeeds");
    assert_eq!(m.vertex_count(), 4);
    assert_eq!(m.face_count(), 4);
    m.validate().expect("valid topology");
}
