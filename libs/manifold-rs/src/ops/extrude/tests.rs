use super::*;
use crate::core::cross_section::CrossSection;

#[test]
fn test_extrude_stub() {
    let cs = CrossSection::square(DVec2::new(10.0, 10.0), false);
    let res = linear_extrude(&cs, 10.0, 0.0, 1, false, DVec2::ONE);
    assert!(res.is_err()); // Currently stubbed
}
