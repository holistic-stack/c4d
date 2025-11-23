use super::*;
use crate::core::cross_section::CrossSection;
use glam::DVec2;

#[test]
fn test_offset_simple() {
    let cs = CrossSection::square(DVec2::new(10.0, 10.0), true);
    // Offset by 1.0
    // We use a small miter limit to avoid excessive expansion if mitered
    let res = offset(&cs, 1.0, JoinType::Round, 2.0).expect("offset succeeds");

    // Square 10x10 (-5 to 5).
    // Offset 1 -> 12x12 (-6 to 6).
    // Check bounding box or points.
    // If something failed silently, result might be empty or original.

    if res.contours.is_empty() {
        panic!("Offset returned empty contours");
    }

    let contour = &res.contours[0];
    let mut max_x = -f64::INFINITY;
    for p in contour {
        max_x = max_x.max(p.x);
    }

    // Debug print max_x
    println!("Max X: {}", max_x);

    // Verify expansion
    assert!(max_x > 5.1, "Expected expansion, got max_x={}", max_x);
    // Verify it's close to 6.0 (assuming correct algorithm)
    assert!((max_x - 6.0).abs() < 0.1, "Expected 6.0, got {}", max_x);
}
