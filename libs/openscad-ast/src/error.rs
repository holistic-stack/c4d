//! # AST Errors
//!
//! Error types for AST generation.

use thiserror::Error;

/// Errors that can occur during AST generation.
#[derive(Debug, Clone, Error)]
pub enum AstError {
    /// Parse error from the parser.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Invalid CST structure.
    #[error("Invalid CST: {0}")]
    InvalidCst(String),

    /// Unsupported node type.
    #[error("Unsupported node type: {0}")]
    UnsupportedNode(String),

    /// Invalid expression.
    #[error("Invalid expression: {0}")]
    InvalidExpression(String),

    /// Invalid number format.
    #[error("Invalid number: {0}")]
    InvalidNumber(String),
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AstError::ParseError("test".to_string());
        assert!(err.to_string().contains("Parse error"));
    }
}
