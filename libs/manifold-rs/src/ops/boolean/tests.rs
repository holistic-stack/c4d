use crate::{Manifold, BooleanOp, primitives::cube::cube, Vec3};

#[test]
fn test_union_disjoint() {
    let m1 = cube(Vec3::new(1.0, 1.0, 1.0), false).unwrap();
    // Disjoint cube
    let mut m2 = cube(Vec3::new(1.0, 1.0, 1.0), false).unwrap();
    m2.transform(glam::DMat4::from_translation(Vec3::new(2.0, 0.0, 0.0)));

    let m3 = m1.boolean(&m2, BooleanOp::Union).expect("union succeeds");
    assert_eq!(m3.vertex_count(), 16);
    assert_eq!(m3.face_count(), 24);
    m3.validate().expect("valid topology");
}
