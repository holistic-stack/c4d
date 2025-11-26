//! # Mesh Errors
//!
//! Error types for mesh generation operations.

use openscad_ast::Span;
use thiserror::Error;

/// Errors that can occur during mesh generation.
#[derive(Debug, Error)]
pub enum MeshError {
    /// Evaluation error from the eval layer
    #[error("Evaluation error: {0}")]
    EvalError(#[from] openscad_eval::EvalError),

    /// Invalid mesh topology
    #[error("Invalid topology: {message}")]
    InvalidTopology { message: String, span: Option<Span> },

    /// Degenerate geometry
    #[error("Degenerate geometry: {message}")]
    DegenerateGeometry { message: String, span: Option<Span> },

    /// Boolean operation failed
    #[error("Boolean operation failed: {message}")]
    BooleanFailed { message: String, span: Option<Span> },

    /// Unsupported operation
    #[error("Unsupported: {message}")]
    Unsupported { message: String, span: Option<Span> },

    /// Mesh validation failed
    #[error("Validation failed: {message}")]
    ValidationFailed { message: String },

    /// Too many vertices
    #[error("Too many vertices: {count} (max: {max})")]
    TooManyVertices { count: usize, max: usize },

    /// Too many triangles
    #[error("Too many triangles: {count} (max: {max})")]
    TooManyTriangles { count: usize, max: usize },
}

impl MeshError {
    /// Creates an invalid topology error.
    pub fn invalid_topology(message: impl Into<String>, span: Option<Span>) -> Self {
        Self::InvalidTopology {
            message: message.into(),
            span,
        }
    }

    /// Creates a degenerate geometry error.
    pub fn degenerate(message: impl Into<String>, span: Option<Span>) -> Self {
        Self::DegenerateGeometry {
            message: message.into(),
            span,
        }
    }

    /// Creates a boolean operation failed error.
    pub fn boolean_failed(message: impl Into<String>, span: Option<Span>) -> Self {
        Self::BooleanFailed {
            message: message.into(),
            span,
        }
    }

    /// Creates an unsupported operation error.
    pub fn unsupported(message: impl Into<String>, span: Option<Span>) -> Self {
        Self::Unsupported {
            message: message.into(),
            span,
        }
    }
}
