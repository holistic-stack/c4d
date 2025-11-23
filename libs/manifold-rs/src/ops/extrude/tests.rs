use super::*;
use crate::core::cross_section::CrossSection;
use glam::DVec2;

#[test]
fn test_linear_extrude_simple() {
    let cs = CrossSection::square(DVec2::new(10.0, 10.0), false);
    let res = linear_extrude(&cs, 10.0, 0.0, 1, false, DVec2::ONE).expect("extrude succeeds");

    assert_eq!(res.vertex_count(), 8); // 4 bottom, 4 top
    // 1 slice -> side faces: 4 quads -> 8 triangles.
    // Caps: 2 triangles each -> 4 triangles.
    // Total faces: 12 triangles.
    assert_eq!(res.face_count(), 12);
    res.validate().expect("valid topology");
}

#[test]
fn test_linear_extrude_twist() {
    let cs = CrossSection::square(DVec2::new(10.0, 10.0), true);
    // Twist 90 degrees with 2 slices
    let res = linear_extrude(&cs, 10.0, 90.0, 2, true, DVec2::ONE).expect("extrude succeeds");

    // Vertices: 3 levels * 4 points = 12.
    assert_eq!(res.vertex_count(), 12);
    // Side faces: 2 slices * 4 sides * 2 tris = 16.
    // Caps: 2 * 2 = 4.
    // Total: 20.
    assert_eq!(res.face_count(), 20);
    res.validate().expect("valid topology");
}
