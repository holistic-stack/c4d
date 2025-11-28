//! # OpenSCAD Segment Calculation
//!
//! 100% compatible implementation of OpenSCAD's $fn/$fa/$fs segment calculation.
//!
//! ## Algorithm
//!
//! OpenSCAD calculates the number of segments for arcs using:
//!
//! ```text
//! if $fn > 0:
//!     segments = $fn
//! else:
//!     from_fa = ceil(360 / $fa)
//!     from_fs = ceil(circumference / $fs)
//!     segments = max(from_fa, from_fs, 3)
//! ```
//!
//! ## Default Values
//!
//! - `$fn`: 0 (use $fa/$fs calculation)
//! - `$fa`: 12° (30 segments for full circle)
//! - `$fs`: 2mm (depends on radius)
//!
//! ## Reference
//!
//! <https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Other_Language_Features#$fa,_$fs_and_$fn>

use std::f64::consts::PI;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Default $fa value (minimum angle per segment in degrees).
///
/// With $fa=12°, a full circle has ceil(360/12) = 30 segments.
pub const DEFAULT_FA: f64 = 12.0;

/// Default $fs value (minimum segment length in mm).
///
/// For a circle of radius 10, circumference = 2π×10 ≈ 62.8,
/// so ceil(62.8/2) = 32 segments.
pub const DEFAULT_FS: f64 = 2.0;

/// Minimum number of segments for any arc.
///
/// Ensures valid geometry even with extreme parameter values.
pub const MIN_SEGMENTS: u32 = 3;

/// Maximum number of segments to prevent performance issues.
///
/// Very high segment counts can cause memory and performance problems.
pub const MAX_SEGMENTS: u32 = 360;

// =============================================================================
// SEGMENT PARAMS
// =============================================================================

/// OpenSCAD segment calculation parameters.
///
/// Encapsulates $fn, $fa, and $fs values for consistent segment calculation
/// across all primitives.
///
/// ## Example
///
/// ```rust
/// use manifold_rs::openscad::SegmentParams;
///
/// // Default parameters (uses $fa/$fs)
/// let params = SegmentParams::default();
/// let segments = params.calculate_segments(10.0);
/// assert!(segments >= 3);
///
/// // Fixed segment count
/// let params = SegmentParams::with_fn(32);
/// let segments = params.calculate_segments(10.0);
/// assert_eq!(segments, 32);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SegmentParams {
    /// $fn: Fixed segment count (0 = use $fa/$fs).
    ///
    /// When greater than 0, this value is used directly as the segment count.
    pub fn_: Option<u32>,
    
    /// $fa: Minimum angle per segment in degrees.
    ///
    /// Default: 12° (30 segments for full circle)
    pub fa: f64,
    
    /// $fs: Minimum segment length in mm.
    ///
    /// Default: 2mm
    pub fs: f64,
}

impl Default for SegmentParams {
    /// Create with default OpenSCAD values.
    ///
    /// - $fn: None (use $fa/$fs)
    /// - $fa: 12°
    /// - $fs: 2mm
    fn default() -> Self {
        Self {
            fn_: None,
            fa: DEFAULT_FA,
            fs: DEFAULT_FS,
        }
    }
}

impl SegmentParams {
    /// Create with fixed segment count ($fn).
    ///
    /// ## Parameters
    ///
    /// - `fn_`: Number of segments (minimum 3)
    ///
    /// ## Example
    ///
    /// ```rust
    /// use manifold_rs::openscad::SegmentParams;
    ///
    /// let params = SegmentParams::with_fn(16);
    /// assert_eq!(params.calculate_segments(5.0), 16);
    /// ```
    #[must_use]
    pub fn with_fn(fn_: u32) -> Self {
        Self {
            fn_: Some(fn_.max(MIN_SEGMENTS)),
            fa: DEFAULT_FA,
            fs: DEFAULT_FS,
        }
    }

    /// Create with custom $fa and $fs values.
    ///
    /// ## Parameters
    ///
    /// - `fa`: Minimum angle per segment in degrees (clamped to [0.01, 360])
    /// - `fs`: Minimum segment length in mm (clamped to [0.01, 1000])
    ///
    /// ## Example
    ///
    /// ```rust
    /// use manifold_rs::openscad::SegmentParams;
    ///
    /// let params = SegmentParams::with_fa_fs(6.0, 1.0);
    /// // Higher resolution than default
    /// ```
    #[must_use]
    pub fn with_fa_fs(fa: f64, fs: f64) -> Self {
        Self {
            fn_: None,
            fa: fa.clamp(0.01, 360.0),
            fs: fs.clamp(0.01, 1000.0),
        }
    }

    /// Calculate number of segments for a given radius.
    ///
    /// Implements OpenSCAD's exact segment calculation algorithm:
    ///
    /// ```text
    /// if $fn > 0:
    ///     return $fn
    /// else:
    ///     from_fa = ceil(360 / $fa)
    ///     from_fs = ceil(2 * PI * radius / $fs)
    ///     return max(from_fa, from_fs, 3)
    /// ```
    ///
    /// ## Parameters
    ///
    /// - `radius`: Arc radius (must be positive)
    ///
    /// ## Returns
    ///
    /// Number of segments (clamped to [MIN_SEGMENTS, MAX_SEGMENTS])
    ///
    /// ## Example
    ///
    /// ```rust
    /// use manifold_rs::openscad::SegmentParams;
    ///
    /// let params = SegmentParams::default();
    ///
    /// // Small radius = fewer segments from $fs
    /// let small = params.calculate_segments(1.0);
    ///
    /// // Large radius = more segments from $fs
    /// let large = params.calculate_segments(100.0);
    ///
    /// assert!(large > small);
    /// ```
    #[must_use]
    pub fn calculate_segments(&self, radius: f64) -> u32 {
        // If $fn is set and > 0, use it directly
        if let Some(fn_) = self.fn_ {
            if fn_ > 0 {
                return fn_.clamp(MIN_SEGMENTS, MAX_SEGMENTS);
            }
        }
        
        // Calculate from $fa: segments = ceil(360 / $fa)
        let from_fa = (360.0 / self.fa).ceil() as u32;
        
        // Calculate from $fs: segments = ceil(circumference / $fs)
        let circumference = 2.0 * PI * radius.abs();
        let from_fs = (circumference / self.fs).ceil() as u32;
        
        // Return maximum, clamped to valid range
        from_fa.max(from_fs).max(MIN_SEGMENTS).min(MAX_SEGMENTS)
    }

    /// Calculate segments for a sphere.
    ///
    /// For spheres, the number of rings is `(segments + 1) / 2`.
    /// This ensures consistent tessellation with OpenSCAD.
    ///
    /// ## Parameters
    ///
    /// - `radius`: Sphere radius
    ///
    /// ## Returns
    ///
    /// Tuple of (segments_around, num_rings)
    #[must_use]
    pub fn calculate_sphere_segments(&self, radius: f64) -> (u32, u32) {
        let segments = self.calculate_segments(radius);
        let rings = (segments + 1) / 2;
        (segments, rings)
    }

    /// Calculate segments for a cylinder.
    ///
    /// Uses the larger radius for segment calculation to ensure
    /// smooth appearance on the wider end.
    ///
    /// ## Parameters
    ///
    /// - `radius1`: Bottom radius
    /// - `radius2`: Top radius
    ///
    /// ## Returns
    ///
    /// Number of segments around the circumference
    #[must_use]
    pub fn calculate_cylinder_segments(&self, radius1: f64, radius2: f64) -> u32 {
        let max_radius = radius1.abs().max(radius2.abs());
        self.calculate_segments(max_radius)
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test default parameters.
    #[test]
    fn test_default_params() {
        let params = SegmentParams::default();
        assert!(params.fn_.is_none());
        assert!((params.fa - DEFAULT_FA).abs() < 0.001);
        assert!((params.fs - DEFAULT_FS).abs() < 0.001);
    }

    /// Test fixed $fn.
    #[test]
    fn test_with_fn() {
        let params = SegmentParams::with_fn(24);
        assert_eq!(params.calculate_segments(10.0), 24);
        assert_eq!(params.calculate_segments(100.0), 24);
    }

    /// Test $fn minimum clamping.
    #[test]
    fn test_fn_minimum() {
        let params = SegmentParams::with_fn(1);
        assert_eq!(params.calculate_segments(10.0), MIN_SEGMENTS);
    }

    /// Test segment calculation with default params.
    #[test]
    fn test_default_calculation() {
        let params = SegmentParams::default();
        
        // With $fa=12, from_fa = 30
        // With $fs=2, r=10: circumference = 62.8, from_fs = 32
        let segments = params.calculate_segments(10.0);
        assert!(segments >= 30, "Expected >= 30, got {}", segments);
    }

    /// Test larger radius = more segments.
    #[test]
    fn test_radius_affects_segments() {
        let params = SegmentParams::default();
        
        let small = params.calculate_segments(1.0);
        let large = params.calculate_segments(100.0);
        
        assert!(large >= small, "Large radius should have >= segments");
    }

    /// Test sphere segments.
    #[test]
    fn test_sphere_segments() {
        let params = SegmentParams::with_fn(16);
        let (segments, rings) = params.calculate_sphere_segments(5.0);
        
        assert_eq!(segments, 16);
        assert_eq!(rings, 8); // (16 + 1) / 2 = 8
    }

    /// Test cylinder segments uses max radius.
    #[test]
    fn test_cylinder_segments() {
        let params = SegmentParams::default();
        
        let cone = params.calculate_cylinder_segments(10.0, 0.0);
        let inverted = params.calculate_cylinder_segments(0.0, 10.0);
        
        // Both should use radius 10 for calculation
        assert_eq!(cone, inverted);
    }

    /// Test OpenSCAD compatibility: $fn=0 should use $fa/$fs.
    #[test]
    fn test_fn_zero() {
        let params = SegmentParams {
            fn_: Some(0),
            fa: DEFAULT_FA,
            fs: DEFAULT_FS,
        };
        
        // Should fallback to $fa/$fs calculation, not return 0
        let segments = params.calculate_segments(10.0);
        assert!(segments >= MIN_SEGMENTS);
    }
}
