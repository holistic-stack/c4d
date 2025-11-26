//! # Config Crate
//!
//! Centralized configuration constants for the Rust OpenSCAD pipeline.
//! All magic numbers and tunable parameters are defined here to ensure
//! consistency across crates and easy configuration management.
//!
//! ## Usage
//!
//! ```rust
//! use config::constants::{EPSILON, DEFAULT_FN, DEFAULT_FA, DEFAULT_FS};
//!
//! // Use EPSILON for floating-point comparisons
//! let value: f64 = 0.00000000001; // 1e-11, smaller than EPSILON (1e-10)
//! let is_zero = value.abs() < EPSILON;
//! assert!(is_zero);
//!
//! // Use resolution defaults for tessellation
//! let fn_override = 0.0;
//! let segments = if fn_override > 0.0 { fn_override } else { DEFAULT_FN };
//! assert_eq!(segments, DEFAULT_FN);
//! ```
//!
//! ## Design Principles
//!
//! - **Single Source of Truth**: All constants defined once, used everywhere
//! - **Browser-Safe**: No platform-specific values
//! - **OpenSCAD Compatible**: Defaults match OpenSCAD behavior
//! - **Well-Documented**: Every constant has clear documentation

pub mod constants;

#[cfg(test)]
mod tests;
