//! # Resolution Calculation
//!
//! Computes fragment counts for circular shapes using OpenSCAD's $fn/$fa/$fs rules.

use crate::EvaluationContext;
use config::constants::{compute_fragments, MAX_FRAGMENTS, MIN_FRAGMENTS};

/// Computes the number of segments for a circular shape.
///
/// Implements OpenSCAD's resolution formula:
/// - If $fn > 0: use $fn (clamped to MIN_FRAGMENTS..MAX_FRAGMENTS)
/// - Otherwise: ceil(min(360/$fa, 2*PI*r/$fs)) clamped to MIN_FRAGMENTS..MAX_FRAGMENTS
///
/// # Arguments
///
/// * `radius` - The radius of the circular shape
/// * `ctx` - The evaluation context containing $fn, $fa, $fs values
///
/// # Returns
///
/// The number of segments to use for tessellation.
///
/// # Example
///
/// ```rust
/// use openscad_eval::{EvaluationContext, resolution::compute_segments};
///
/// let mut ctx = EvaluationContext::new();
/// ctx.set_fn(32.0);
/// let segments = compute_segments(10.0, &ctx);
/// assert_eq!(segments, 32);
/// ```
pub fn compute_segments(radius: f64, ctx: &EvaluationContext) -> u32 {
    compute_fragments(radius, ctx.fn_value(), ctx.fa_value(), ctx.fs_value())
}

/// Computes segments with an explicit $fn override.
///
/// Used when a primitive has a local $fn parameter.
///
/// # Example
///
/// ```rust
/// use openscad_eval::{EvaluationContext, resolution::compute_segments_with_override};
///
/// let ctx = EvaluationContext::new();
/// let segments = compute_segments_with_override(10.0, Some(64.0), &ctx);
/// assert_eq!(segments, 64);
/// ```
pub fn compute_segments_with_override(
    radius: f64,
    fn_override: Option<f64>,
    ctx: &EvaluationContext,
) -> u32 {
    let fn_value = fn_override.unwrap_or(ctx.fn_value());
    compute_fragments(radius, fn_value, ctx.fa_value(), ctx.fs_value())
}

/// Validates that segments are within acceptable range.
///
/// Returns the clamped value and whether clamping occurred.
pub fn validate_segments(segments: u32) -> (u32, bool) {
    let clamped = segments.clamp(MIN_FRAGMENTS, MAX_FRAGMENTS);
    (clamped, clamped != segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_segments_with_fn() {
        let mut ctx = EvaluationContext::new();
        ctx.set_fn(32.0);
        let segments = compute_segments(10.0, &ctx);
        assert_eq!(segments, 32);
    }

    #[test]
    fn test_compute_segments_without_fn() {
        let ctx = EvaluationContext::new();
        // With default $fa=12, $fs=2, radius=10
        let segments = compute_segments(10.0, &ctx);
        // Should be calculated from $fa/$fs
        assert!(segments >= MIN_FRAGMENTS);
        assert!(segments <= MAX_FRAGMENTS);
    }

    #[test]
    fn test_compute_segments_clamps_min() {
        let mut ctx = EvaluationContext::new();
        ctx.set_fn(1.0); // Too few
        let segments = compute_segments(10.0, &ctx);
        assert_eq!(segments, MIN_FRAGMENTS);
    }

    #[test]
    fn test_compute_segments_clamps_max() {
        let mut ctx = EvaluationContext::new();
        ctx.set_fn(100000.0); // Too many
        let segments = compute_segments(10.0, &ctx);
        assert_eq!(segments, MAX_FRAGMENTS);
    }

    #[test]
    fn test_compute_segments_with_override() {
        let ctx = EvaluationContext::new();
        let segments = compute_segments_with_override(10.0, Some(48.0), &ctx);
        assert_eq!(segments, 48);
    }

    #[test]
    fn test_compute_segments_override_none() {
        let mut ctx = EvaluationContext::new();
        ctx.set_fn(24.0);
        let segments = compute_segments_with_override(10.0, None, &ctx);
        assert_eq!(segments, 24);
    }

    #[test]
    fn test_validate_segments_in_range() {
        let (result, clamped) = validate_segments(32);
        assert_eq!(result, 32);
        assert!(!clamped);
    }

    #[test]
    fn test_validate_segments_too_low() {
        let (result, clamped) = validate_segments(2);
        assert_eq!(result, MIN_FRAGMENTS);
        assert!(clamped);
    }

    #[test]
    fn test_validate_segments_too_high() {
        let (result, clamped) = validate_segments(10000);
        assert_eq!(result, MAX_FRAGMENTS);
        assert!(clamped);
    }
}
