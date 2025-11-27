//! # Evaluation Errors
//!
//! Error types for AST evaluation.

use thiserror::Error;

/// Errors that can occur during evaluation.
#[derive(Debug, Clone, Error)]
pub enum EvalError {
    /// Parse error from earlier stage.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Type mismatch in operation.
    #[error("Type error: {0}")]
    TypeError(String),

    /// Unknown module or function.
    #[error("Unknown identifier: {0}")]
    UnknownIdentifier(String),

    /// Invalid argument.
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Wrong number of arguments.
    #[error("Wrong number of arguments for {0}: expected {1}, got {2}")]
    WrongArgCount(String, usize, usize),

    /// Division by zero.
    #[error("Division by zero")]
    DivisionByZero,

    /// Invalid range.
    #[error("Invalid range: {0}")]
    InvalidRange(String),
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = EvalError::TypeError("expected number".to_string());
        assert!(err.to_string().contains("Type error"));
    }
}
