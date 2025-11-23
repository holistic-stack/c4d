use super::*;
use crate::core::cross_section::CrossSection;
use glam::DVec2;

#[test]
fn test_revolve_simple() {
    let cs = CrossSection::square(DVec2::new(10.0, 10.0), false);
    // Revolve 360 with 4 segments
    let res = rotate_extrude(&cs, 360.0, 0, 4).expect("revolve succeeds");

    // 4 segments -> 4 profiles.
    // Closed -> 4 profiles.
    // Square has 4 points.
    // Vertices: 4 * 4 = 16.
    assert_eq!(res.vertex_count(), 16);

    // Faces: 4 segments * 4 sides * 2 tris = 32 triangles.
    assert_eq!(res.face_count(), 32);
    res.validate().expect("valid topology");
}

#[test]
fn test_revolve_partial() {
    let cs = CrossSection::square(DVec2::new(10.0, 10.0), false);
    // Revolve 90 with 4 segments
    let res = rotate_extrude(&cs, 90.0, 0, 4).expect("revolve succeeds");

    // Partial (open).
    // steps = 4 * (90/360) = 1.
    // But code: ceil(4 * 0.25) = 1.
    // num_profiles = steps + 1 = 2.
    // Vertices: 2 * 4 = 8.
    assert_eq!(res.vertex_count(), 8);

    // Faces: 1 step * 4 sides * 2 tris = 8.
    // Caps: 2 caps * 2 tris = 4.
    // Total: 12.
    assert_eq!(res.face_count(), 12);
    res.validate().expect("valid topology");
}
