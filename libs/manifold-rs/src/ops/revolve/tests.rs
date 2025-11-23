use super::*;
use crate::core::cross_section::CrossSection;
use glam::DVec2;

#[test]
fn test_revolve_stub() {
    let cs = CrossSection::square(DVec2::new(10.0, 10.0), false);
    let res = rotate_extrude(&cs, 360.0, 10, 30);
    assert!(res.is_err()); // Currently stubbed
}
