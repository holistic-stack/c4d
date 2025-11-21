//! Tests for the centralized configuration constants.

use super::*;

/// Ensures default constants are sane and positive.
///
/// # Examples
/// ```
/// use config::constants::GlobalConfig;
/// let cfg = GlobalConfig::default();
/// assert!(cfg.tolerance > 0.0);
/// ```
#[test]
fn default_constants_are_valid() {
    let cfg = GlobalConfig::default();
    assert!(cfg.tolerance > 0.0);
    assert!(cfg.default_segments >= 3);
}

/// Validates the builder rejects invalid values.
///
/// # Examples
/// ```
/// use config::constants::GlobalConfig;
/// assert!(GlobalConfig::new(0.0, 24).is_err());
/// ```
#[test]
fn new_validates_inputs() {
    assert_eq!(
        GlobalConfig::new(0.0, 24).unwrap_err(),
        ConfigError::InvalidTolerance(0.0)
    );
    assert_eq!(
        GlobalConfig::new(1.0e-9, 2).unwrap_err(),
        ConfigError::InvalidSegments(2)
    );
}
