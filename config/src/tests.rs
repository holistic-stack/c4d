//! # Tests for Config Constants
//!
//! Unit tests verifying the correctness of configuration constants
//! and helper functions.

use crate::constants::*;

// =============================================================================
// PRECISION TESTS
// =============================================================================

#[test]
fn test_epsilon_is_positive() {
    assert!(EPSILON > 0.0, "EPSILON must be positive");
}

#[test]
fn test_epsilon_is_small() {
    assert!(EPSILON < 1e-6, "EPSILON should be small for precision");
}

#[test]
fn test_vertex_merge_epsilon_larger_than_epsilon() {
    assert!(
        VERTEX_MERGE_EPSILON >= EPSILON,
        "VERTEX_MERGE_EPSILON should be >= EPSILON"
    );
}

#[test]
fn test_coordinate_scale_is_large() {
    assert!(
        COORDINATE_SCALE >= 1e4,
        "COORDINATE_SCALE should be large enough for precision"
    );
}

// =============================================================================
// RESOLUTION TESTS
// =============================================================================

#[test]
fn test_default_fn_is_zero() {
    // OpenSCAD default: $fn = 0 means use $fa/$fs calculation
    assert_eq!(DEFAULT_FN, 0.0);
}

#[test]
fn test_default_fa_matches_openscad() {
    // OpenSCAD default: $fa = 12 degrees
    assert_eq!(DEFAULT_FA, 12.0);
}

#[test]
fn test_default_fs_matches_openscad() {
    // OpenSCAD default: $fs = 2 units
    assert_eq!(DEFAULT_FS, 2.0);
}

#[test]
fn test_min_fragments_at_least_three() {
    // A circle needs at least 3 points to form a polygon
    assert!(MIN_FRAGMENTS >= 3);
}

#[test]
fn test_max_fragments_reasonable() {
    // Should be large enough for smooth circles but not excessive
    assert!(MAX_FRAGMENTS >= 100);
    assert!(MAX_FRAGMENTS <= 10000);
}

// =============================================================================
// COMPUTE_FRAGMENTS TESTS
// =============================================================================

#[test]
fn test_compute_fragments_with_fn_override() {
    // When $fn > 0, use it directly
    let fragments = compute_fragments(10.0, 32.0, DEFAULT_FA, DEFAULT_FS);
    assert_eq!(fragments, 32);
}

#[test]
fn test_compute_fragments_clamps_to_min() {
    // Even with $fn = 1, should clamp to MIN_FRAGMENTS
    let fragments = compute_fragments(10.0, 1.0, DEFAULT_FA, DEFAULT_FS);
    assert_eq!(fragments, MIN_FRAGMENTS);
}

#[test]
fn test_compute_fragments_clamps_to_max() {
    // With very high $fn, should clamp to MAX_FRAGMENTS
    let fragments = compute_fragments(10.0, 100000.0, DEFAULT_FA, DEFAULT_FS);
    assert_eq!(fragments, MAX_FRAGMENTS);
}

#[test]
fn test_compute_fragments_without_fn_small_radius() {
    // Small radius with default $fa/$fs
    let fragments = compute_fragments(1.0, 0.0, DEFAULT_FA, DEFAULT_FS);
    // Should be clamped to MIN_FRAGMENTS for very small circles
    assert!(fragments >= MIN_FRAGMENTS);
}

#[test]
fn test_compute_fragments_without_fn_large_radius() {
    // Large radius with default $fa/$fs
    let fragments = compute_fragments(100.0, 0.0, DEFAULT_FA, DEFAULT_FS);
    // Should produce more fragments for larger circles
    assert!(fragments > MIN_FRAGMENTS);
}

#[test]
fn test_compute_fragments_fa_dominates() {
    // With very large $fs, $fa should dominate
    // But min() takes the smaller of the two, so with large $fs:
    // from_angle = 360 / 12 = 30
    // from_size = 2*PI*10 / 1000 = 0.06 -> ceil = 1
    // min(30, 1) = 1, clamped to MIN_FRAGMENTS = 5
    let fragments = compute_fragments(10.0, 0.0, 12.0, 1000.0);
    assert_eq!(fragments, MIN_FRAGMENTS);
}

#[test]
fn test_compute_fragments_fs_dominates() {
    // With very small $fa, $fs should dominate
    let fragments = compute_fragments(10.0, 0.0, 0.1, 2.0);
    // 2*PI*10 / 2 ≈ 31.4, ceil = 32 fragments from $fs
    let expected = ((2.0 * std::f64::consts::PI * 10.0) / 2.0).ceil() as u32;
    assert_eq!(fragments, expected);
}

// =============================================================================
// APPROX_EQUAL TESTS
// =============================================================================

#[test]
fn test_approx_equal_same_values() {
    assert!(approx_equal(1.0, 1.0));
    assert!(approx_equal(0.0, 0.0));
    assert!(approx_equal(-5.5, -5.5));
}

#[test]
fn test_approx_equal_within_epsilon() {
    let small_diff = EPSILON / 2.0;
    assert!(approx_equal(1.0, 1.0 + small_diff));
    assert!(approx_equal(1.0, 1.0 - small_diff));
}

#[test]
fn test_approx_equal_outside_epsilon() {
    let large_diff = EPSILON * 2.0;
    assert!(!approx_equal(1.0, 1.0 + large_diff));
    assert!(!approx_equal(1.0, 1.0 - large_diff));
}

#[test]
fn test_approx_equal_different_values() {
    assert!(!approx_equal(1.0, 2.0));
    assert!(!approx_equal(0.0, 1.0));
}

// =============================================================================
// APPROX_ZERO TESTS
// =============================================================================

#[test]
fn test_approx_zero_exact_zero() {
    assert!(approx_zero(0.0));
}

#[test]
fn test_approx_zero_within_epsilon() {
    let small = EPSILON / 2.0;
    assert!(approx_zero(small));
    assert!(approx_zero(-small));
}

#[test]
fn test_approx_zero_outside_epsilon() {
    let large = EPSILON * 2.0;
    assert!(!approx_zero(large));
    assert!(!approx_zero(-large));
}

#[test]
fn test_approx_zero_non_zero_values() {
    assert!(!approx_zero(1.0));
    assert!(!approx_zero(-1.0));
    assert!(!approx_zero(0.1));
}

// =============================================================================
// LIMIT TESTS
// =============================================================================

#[test]
fn test_max_recursion_depth_reasonable() {
    // Should be large enough for complex models but not infinite
    assert!(MAX_RECURSION_DEPTH >= 100);
    assert!(MAX_RECURSION_DEPTH <= 10000);
}

#[test]
fn test_max_vertices_reasonable() {
    // Should allow complex models but prevent memory exhaustion
    assert!(MAX_VERTICES >= 1_000_000);
}

#[test]
fn test_max_triangles_reasonable() {
    // Should allow complex models but prevent memory exhaustion
    assert!(MAX_TRIANGLES >= 1_000_000);
}

// =============================================================================
// COLOR TESTS
// =============================================================================

#[test]
fn test_default_color_valid_rgba() {
    for component in DEFAULT_COLOR.iter() {
        assert!(*component >= 0.0 && *component <= 1.0);
    }
}

#[test]
fn test_default_color_is_opaque() {
    // Alpha channel should be 1.0 (fully opaque)
    assert_eq!(DEFAULT_COLOR[3], 1.0);
}
