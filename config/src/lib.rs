//! Shared configuration crate holding constants used across the workspace.
//!
//! The `constants` module exposes strongly typed settings so downstream crates
//! avoid sprinkling magic numbers. Example:
//!
//! ```
//! use config::constants::{GlobalConfig, EPSILON_TOLERANCE};
//!
//! let cfg = GlobalConfig::default();
//! assert_eq!(cfg.tolerance, EPSILON_TOLERANCE);
//! ```

pub mod constants;

pub use constants::{
    GlobalConfig, DEFAULT_SEGMENTS, EPSILON_TOLERANCE, MAX_LSP_DOCUMENTS, STACKER_STACK_SIZE_BYTES,
};
