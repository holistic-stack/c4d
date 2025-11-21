//! Tests covering kernel configuration behavior.

use super::*;

#[test]
fn default_matches_constants() {
    let cfg = KernelConfig::default();
    assert_eq!(cfg.tolerance, EPSILON_TOLERANCE);
    assert_eq!(cfg.default_segments, DEFAULT_SEGMENTS);
}

#[test]
fn builder_validates_input() {
    let err = KernelConfig::new(0.0, 24).unwrap_err();
    assert_eq!(err, KernelConfigError(ConfigError::InvalidTolerance(0.0)));
}
