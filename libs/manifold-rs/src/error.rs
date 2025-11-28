//! # Error Types
//!
//! Error types for manifold operations. All errors are explicit and provide
//! clear debugging information.
//!
//! ## Error Policy
//!
//! - NO fallback mechanisms when operations fail
//! - All failures throw explicit errors
//! - Errors include context for debugging

use thiserror::Error;

// =============================================================================
// ERROR TYPES
// =============================================================================

/// Errors that can occur during manifold operations.
///
/// ## Example
///
/// ```rust
/// use manifold_rs::{render, ManifoldError};
///
/// match render("invalid syntax {") {
///     Ok(mesh) => println!("Success: {} vertices", mesh.vertex_count()),
///     Err(ManifoldError::EvalError(msg)) => eprintln!("Eval error: {}", msg),
///     Err(e) => eprintln!("Other error: {}", e),
/// }
/// ```
#[derive(Error, Debug)]
pub enum ManifoldError {
    /// Error during OpenSCAD source evaluation.
    ///
    /// Contains the error message from openscad-eval.
    #[error("Evaluation error: {0}")]
    EvalError(String),
    
    /// Error during mesh geometry generation.
    ///
    /// Contains description of what went wrong.
    #[error("Geometry error: {0}")]
    GeometryError(String),
    
    /// Error during boolean operation.
    ///
    /// Contains the operation name and error details.
    #[error("Boolean operation '{operation}' failed: {message}")]
    BooleanError {
        /// Name of the boolean operation (union, difference, intersection)
        operation: String,
        /// Error message
        message: String,
    },
    
    /// Error during mesh validation.
    ///
    /// The mesh is not manifold (watertight).
    #[error("Mesh is not manifold: {0}")]
    NonManifoldError(String),
    
    /// Error during 2D polygon operation.
    ///
    /// Contains the operation name and error details.
    #[error("2D operation '{operation}' failed: {message}")]
    CrossSectionError {
        /// Name of the 2D operation
        operation: String,
        /// Error message
        message: String,
    },
    
    /// Invalid segment parameters.
    ///
    /// Contains the invalid parameter values.
    #[error("Invalid segment parameters: {0}")]
    InvalidSegmentParams(String),
}

// =============================================================================
// RESULT TYPE ALIAS
// =============================================================================

/// Result type alias for manifold operations.
///
/// ## Example
///
/// ```rust
/// use manifold_rs::error::ManifoldResult;
/// use manifold_rs::Mesh;
///
/// fn create_mesh() -> ManifoldResult<Mesh> {
///     // ... mesh creation logic
///     # Ok(Mesh::new())
/// }
/// ```
pub type ManifoldResult<T> = Result<T, ManifoldError>;

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test error display messages.
    #[test]
    fn test_error_display() {
        let eval_err = ManifoldError::EvalError("parse failed".to_string());
        assert!(eval_err.to_string().contains("Evaluation error"));
        
        let bool_err = ManifoldError::BooleanError {
            operation: "union".to_string(),
            message: "degenerate geometry".to_string(),
        };
        assert!(bool_err.to_string().contains("union"));
        assert!(bool_err.to_string().contains("degenerate"));
    }

    /// Test error types are Send + Sync for async compatibility.
    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ManifoldError>();
    }
}
