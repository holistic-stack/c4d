//! Centralized configuration for geometry operations
//! 
//! All magic numbers and tunable parameters live here.
//! This module provides constants and configuration functions
//! used throughout the manifold-rs library.

use std::sync::OnceLock;

/// Geometric tolerance for numerical operations
/// This value is used for floating-point comparisons and
/// determines when two values are considered equal.
pub const EPSILON: f64 = 1e-9;

/// Default number of segments for curved primitives
/// This is used when no explicit resolution is specified
/// for spheres, cylinders, and other curved surfaces.
pub const DEFAULT_SEGMENTS: u32 = 32;

/// Default number of faces for sphere tessellation
/// This corresponds to the $fn special variable in OpenSCAD
pub const DEFAULT_FN: u32 = 20;

/// Default minimum angle for sphere tessellation
/// This corresponds to the $fa special variable in OpenSCAD
pub const DEFAULT_FA: f64 = 12.0;

/// Default minimum size for sphere tessellation  
/// This corresponds to the $fs special variable in OpenSCAD
pub const DEFAULT_FS: f64 = 2.0;

/// Scaling factor for 2D operations (Clipper2 integration)
/// When converting f64 coordinates to i64 for Clipper2 operations,
/// we multiply by this scale factor to maintain precision.
pub const CLIPPER_SCALE: i64 = 1_000_000;

/// Maximum recursion depth for boolean operations
/// This prevents stack overflow in complex CSG operations
pub const MAX_RECURSION_DEPTH: u32 = 100;

/// Thread pool size for parallel operations
/// Returns the number of threads to use for parallel operations.
/// Uses available_parallelism() when available, falling back to 4.
pub fn thread_pool_size() -> usize {
    static SIZE: OnceLock<usize> = OnceLock::new();
    *SIZE.get_or_init(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    })
}

/// Precision configuration for geometric operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrecisionConfig {
    /// Tolerance for floating-point comparisons
    pub epsilon: f64,
    /// Scale factor for integer operations
    pub clipper_scale: i64,
}

impl Default for PrecisionConfig {
    fn default() -> Self {
        Self {
            epsilon: EPSILON,
            clipper_scale: CLIPPER_SCALE,
        }
    }
}

impl PrecisionConfig {
    /// Creates a high-precision configuration
    pub fn high_precision() -> Self {
        Self {
            epsilon: 1e-12,
            clipper_scale: 1_000_000_000,
        }
    }
    
    /// Creates a fast, lower-precision configuration
    pub fn fast() -> Self {
        Self {
            epsilon: 1e-6,
            clipper_scale: 100_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precision_config_default() {
        let config = PrecisionConfig::default();
        assert_eq!(config.epsilon, EPSILON);
        assert_eq!(config.clipper_scale, CLIPPER_SCALE);
    }

    #[test]
    fn test_precision_config_high() {
        let config = PrecisionConfig::high_precision();
        assert_eq!(config.epsilon, 1e-12);
        assert_eq!(config.clipper_scale, 1_000_000_000);
    }

    #[test]
    fn test_precision_config_fast() {
        let config = PrecisionConfig::fast();
        assert_eq!(config.epsilon, 1e-6);
        assert_eq!(config.clipper_scale, 100_000);
    }

    #[test]
    fn test_thread_pool_size() {
        let size = thread_pool_size();
        assert!(size > 0);
        assert!(size <= 64); // Reasonable upper bound
    }
}