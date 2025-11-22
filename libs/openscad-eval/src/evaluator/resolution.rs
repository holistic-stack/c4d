//! Shared helpers for translating `$fn`, `$fa`, and `$fs` into fragment counts.
//!
//! OpenSCAD’s official documentation describes the fragment selection logic in
//! terms of these special variables: if `$fn` is positive it sets the fragment
//! count directly (with a floor of 3). Otherwise the runtime chooses the
//! smaller value between `360 / $fa` (angle-based) and `2πr / $fs`
//! (arc-length-based), rounds up, and enforces a lower bound of five fragments.
//! Examples and rationale documented at
//! <https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Other_Language_Features>.

use std::f64::consts::TAU;

/// Minimum fragment count when `$fn` is not specified.
const MIN_AUTO_FRAGMENTS: u32 = 5;
/// Minimum fragment count when `$fn` *is* specified.
const MIN_FN_FRAGMENTS: u32 = 3;

/// Computes the fragment/segment count for a sphere-like primitive.
///
/// # Examples
/// ```
/// use openscad_eval::evaluator::resolution::compute_segments;
/// assert_eq!(compute_segments(10.0, 24, 0.0, 0.0), 24);
/// assert_eq!(compute_segments(5.0, 0, 10.0, 2.0), 16); // min(36, ~15.7) -> ceil(~15.7) = 16
/// ```
#[must_use]
pub fn compute_segments(radius: f64, fn_val: u32, fa_val: f64, fs_val: f64) -> u32 {
    if fn_val > 0 {
        return fn_val.max(MIN_FN_FRAGMENTS);
    }

    let fragments_from_fa = if fa_val > 0.0 {
        360.0 / fa_val
    } else {
        f64::INFINITY
    };

    let circumference = radius.abs() * TAU;
    let fragments_from_fs = if fs_val > 0.0 && circumference > 0.0 {
        circumference / fs_val
    } else {
        f64::INFINITY
    };

    let raw = fragments_from_fa.min(fragments_from_fs);
    let computed = if raw.is_finite() { raw.ceil() as u32 } else { MIN_AUTO_FRAGMENTS };
    computed.max(MIN_AUTO_FRAGMENTS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fn_overrides_limits() {
        assert_eq!(compute_segments(1.0, 24, 0.0, 0.0), 24);
        assert_eq!(compute_segments(1.0, 2, 0.0, 0.0), 3);
    }

    #[test]
    fn fa_fs_paths_match_open_scad_rule() {
        // 360 / 90 = 4, circumference/fs = ~0.0628 -> choose 0.0628 -> ceil -> 1 -> clamp to 5
        assert_eq!(compute_segments(0.01, 0, 90.0, 100.0), 5);
        // radius 10, fs 1 => circumference/fs ~62.8, fa -> 360/6 = 60 => choose 60
        assert_eq!(compute_segments(10.0, 0, 6.0, 1.0), 60);
    }

    #[test]
    fn zero_radius_falls_back_to_fa_only() {
        assert_eq!(compute_segments(0.0, 0, 12.0, 2.0), 30);
    }
}
