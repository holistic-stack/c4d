//! # Mesh Generation Errors
//!
//! Error types for mesh generation.

use thiserror::Error;

/// Errors that can occur during mesh generation.
#[derive(Debug, Clone, Error)]
pub enum MeshError {
    /// Evaluation error.
    #[error("Evaluation error: {0}")]
    EvalError(String),

    /// Unsupported geometry.
    #[error("Unsupported geometry: {0}")]
    UnsupportedGeometry(String),

    /// Invalid geometry parameters.
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),

    /// Boolean operation error.
    #[error("Boolean operation failed: {0}")]
    BooleanError(String),
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = MeshError::UnsupportedGeometry("test".to_string());
        assert!(err.to_string().contains("Unsupported"));
    }
}
