//! Kernel-level configuration helpers building on the shared `config` crate.
//!
//! The module re-exports the workspace configuration so geometry components can
//! stay decoupled from literal constants.

use config::constants::{GlobalConfig, DEFAULT_SEGMENTS, EPSILON_TOLERANCE};

/// Geometry kernel configuration wrapper.
///
/// # Examples
/// ```
/// use manifold_rs::config::KernelConfig;
/// let cfg = KernelConfig::default();
/// assert!(cfg.tolerance > 0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KernelConfig {
    /// Global numeric tolerance forwarded to predicates.
    pub tolerance: f64,
    /// Default segment count for primitives that require subdivision.
    pub default_segments: u32,
}

impl KernelConfig {
    /// Creates a new configuration from explicit values.
    ///
    /// # Examples
    /// ```
    /// use manifold_rs::config::KernelConfig;
    /// let cfg = KernelConfig::new(1.0e-8, 48).unwrap();
    /// assert_eq!(cfg.default_segments, 48);
    /// ```
    pub fn new(tolerance: f64, default_segments: u32) -> Result<Self, KernelConfigError> {
        GlobalConfig::new(tolerance, default_segments)
            .map(|cfg| Self {
                tolerance: cfg.tolerance,
                default_segments: cfg.default_segments,
            })
            .map_err(KernelConfigError)
    }
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            tolerance: EPSILON_TOLERANCE,
            default_segments: DEFAULT_SEGMENTS,
        }
    }
}

/// Error wrapper for invalid kernel configuration.
#[derive(Debug, PartialEq)]
pub struct KernelConfigError(ConfigError);

impl std::fmt::Display for KernelConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for KernelConfigError {}

use config::constants::ConfigError;

#[cfg(test)]
mod tests;
