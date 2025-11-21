//! Centralized configuration values shared across the Rust OpenSCAD pipeline.
//!
//! Each public item in this module documents its purpose and provides a minimal
//! usage example so that downstream crates can remain declarative and avoid
//! scattering literals.

use std::fmt;

/// Numerical tolerance used by geometry kernels.
///
/// # Examples
/// ```
/// use config::constants::EPSILON_TOLERANCE;
/// assert!(EPSILON_TOLERANCE < 1.0e-6);
/// ```
pub const EPSILON_TOLERANCE: f64 = 1.0e-9;

/// Default tessellation segment count for primitives that require angular
/// resolution such as cylinders or spheres.
///
/// # Examples
/// ```
/// use config::constants::DEFAULT_SEGMENTS;
/// assert!(DEFAULT_SEGMENTS >= 12);
/// ```
pub const DEFAULT_SEGMENTS: u32 = 32;

/// Bytes of stack space reserved when growing recursion limits using the
/// `stacker` crate.
///
/// # Examples
/// ```
/// use config::constants::STACKER_STACK_SIZE_BYTES;
/// assert!(STACKER_STACK_SIZE_BYTES >= 1024);
/// ```
pub const STACKER_STACK_SIZE_BYTES: usize = 8 * 1024 * 1024;

/// Maximum number of documents the LSP server keeps in memory simultaneously.
///
/// # Examples
/// ```
/// use config::constants::MAX_LSP_DOCUMENTS;
/// assert!(MAX_LSP_DOCUMENTS >= 16);
/// ```
pub const MAX_LSP_DOCUMENTS: usize = 64;

/// Immutable snapshot of global configuration settings that can be shared
/// between crates.
///
/// # Examples
/// ```
/// use config::constants::GlobalConfig;
/// let config = GlobalConfig::default();
/// assert!(config.tolerance > 0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlobalConfig {
    /// Numeric tolerance propagated into geometry kernels.
    pub tolerance: f64,
    /// Default segment count for primitives that require polygonal subdivision.
    pub default_segments: u32,
}

impl GlobalConfig {
    /// Builds a configuration enforcing strict validation of the supplied
    /// tolerance and default segments.
    ///
    /// # Examples
    /// ```
    /// use config::constants::GlobalConfig;
    /// let cfg = GlobalConfig::new(1.0e-6, 24).expect("valid config");
    /// assert_eq!(cfg.default_segments, 24);
    /// ```
    pub fn new(tolerance: f64, default_segments: u32) -> Result<Self, ConfigError> {
        if tolerance <= 0.0 {
            return Err(ConfigError::InvalidTolerance(tolerance));
        }
        if default_segments < 3 {
            return Err(ConfigError::InvalidSegments(default_segments));
        }
        Ok(Self {
            tolerance,
            default_segments,
        })
    }
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            tolerance: EPSILON_TOLERANCE,
            default_segments: DEFAULT_SEGMENTS,
        }
    }
}

/// Error returned when invalid configuration values are provided.
#[derive(Debug, PartialEq)]
pub enum ConfigError {
    /// Raised when tolerance is zero or negative.
    InvalidTolerance(f64),
    /// Raised when the requested segment count is too small to form a polygon.
    InvalidSegments(u32),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::InvalidTolerance(value) => {
                write!(f, "tolerance must be positive: {value}")
            }
            ConfigError::InvalidSegments(value) => {
                write!(f, "default_segments must be >= 3: {value}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests;
