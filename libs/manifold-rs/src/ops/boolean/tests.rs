use super::*;
use crate::primitives::square::square;
use glam::DVec2;

#[test]
fn test_union_disjoint() {
    let m1 = square(DVec2::new(10.0, 10.0), true).unwrap();
    let mut m2 = square(DVec2::new(10.0, 10.0), true).unwrap();

    // Move m2 so they don't overlap
    for v in &mut m2.vertices {
        v.position.x += 20.0;
    }

    let m3 = boolean(&m1, &m2, BooleanOp::Union).expect("union succeeds");

    assert_eq!(m3.vertex_count(), 8);
    assert_eq!(m3.face_count(), 4);
}

#[test]
fn test_difference_unimplemented() {
    let m1 = square(DVec2::new(10.0, 10.0), true).unwrap();
    let m2 = square(DVec2::new(5.0, 5.0), true).unwrap();

    let result = boolean(&m1, &m2, BooleanOp::Difference);

    assert!(result.is_ok());
    let _m3 = result.unwrap();
}
