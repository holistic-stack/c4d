//! # Evaluation Errors
//!
//! Error types for the OpenSCAD evaluator.

use openscad_ast::{Diagnostic, Span};
use thiserror::Error;

/// Errors that can occur during evaluation.
#[derive(Debug, Error)]
pub enum EvalError {
    /// Parse error from the AST layer (native tree-sitter)
    #[cfg(feature = "native-parser")]
    #[error("Parse error: {0}")]
    ParseError(#[from] openscad_ast::parser::ParseError),

    /// CST parse error (browser/WASM)
    #[error("CST parse error: {0}")]
    CstParseError(#[from] openscad_ast::CstParseError),

    /// Unknown variable reference
    #[error("Unknown variable: {name}")]
    UnknownVariable { name: String, span: Span },

    /// Unknown module/function call
    #[error("Unknown module: {name}")]
    UnknownModule { name: String, span: Span },

    /// Invalid argument
    #[error("Invalid argument: {message}")]
    InvalidArgument { message: String, span: Span },

    /// Type mismatch
    #[error("Type mismatch: {message}")]
    TypeMismatch { message: String, span: Span },

    /// Recursion limit exceeded
    #[error("Recursion limit exceeded")]
    RecursionLimit { span: Span },

    /// Invalid geometry
    #[error("Invalid geometry: {message}")]
    InvalidGeometry { message: String, span: Span },
}

impl EvalError {
    /// Converts this error to a diagnostic.
    pub fn to_diagnostic(&self) -> Diagnostic {
        match self {
            #[cfg(feature = "native-parser")]
            EvalError::ParseError(_) => {
                Diagnostic::error(self.to_string(), Span::default())
            }
            EvalError::CstParseError(_) => {
                Diagnostic::error(self.to_string(), Span::default())
            }
            EvalError::UnknownVariable { name, span } => {
                Diagnostic::error(format!("Unknown variable: {}", name), *span)
            }
            EvalError::UnknownModule { name, span } => {
                Diagnostic::error(format!("Unknown module: {}", name), *span)
            }
            EvalError::InvalidArgument { message, span } => {
                Diagnostic::error(format!("Invalid argument: {}", message), *span)
            }
            EvalError::TypeMismatch { message, span } => {
                Diagnostic::error(format!("Type mismatch: {}", message), *span)
            }
            EvalError::RecursionLimit { span } => {
                Diagnostic::error("Recursion limit exceeded", *span)
            }
            EvalError::InvalidGeometry { message, span } => {
                Diagnostic::error(format!("Invalid geometry: {}", message), *span)
            }
        }
    }

    /// Returns the span associated with this error.
    pub fn span(&self) -> Option<Span> {
        match self {
            #[cfg(feature = "native-parser")]
            EvalError::ParseError(_) => None,
            EvalError::CstParseError(_) => None,
            EvalError::UnknownVariable { span, .. } => Some(*span),
            EvalError::UnknownModule { span, .. } => Some(*span),
            EvalError::InvalidArgument { span, .. } => Some(*span),
            EvalError::TypeMismatch { span, .. } => Some(*span),
            EvalError::RecursionLimit { span } => Some(*span),
            EvalError::InvalidGeometry { span, .. } => Some(*span),
        }
    }
}
