//! Core geometry kernel scaffolding for the Rust OpenSCAD pipeline.
//!
//! This crate exposes typed configuration values and mathematical helpers that
//! will back the future half-edge implementation. The initial vertical slice
//! keeps the API intentionally small:
//!
//! ```
//! use manifold_rs::{KernelConfig, Vec3};
//!
//! let config = KernelConfig::default();
//! let axis: Vec3 = Vec3::new(1.0, 0.0, 0.0);
//! assert!(axis.length() > config.tolerance);
//! ```

pub mod config;
pub mod core;
pub mod error;
pub mod export;
pub mod from_ir;
pub mod manifold;
pub mod primitives;
pub mod transform;

pub use config::KernelConfig;
pub use core::vec3::Vec3;
pub use error::ManifoldError;
pub use export::MeshBuffers;
pub use from_ir::from_source;
pub use manifold::Manifold;
pub use transform::apply_transform;

/// Returns the default kernel configuration.
///
/// # Examples
/// ```
/// use manifold_rs::default_config;
/// let config = default_config();
/// assert!(config.default_segments >= 3);
/// ```
pub fn default_config() -> KernelConfig {
    KernelConfig::default()
}

#[cfg(test)]
mod tests {
    //! Tests the crate-level helpers to guarantee re-export stability.

    use super::*;

    #[test]
    fn default_config_uses_positive_values() {
        let config = default_config();
        assert!(config.default_segments >= 3);
        assert!(config.tolerance > 0.0);
    }
}
