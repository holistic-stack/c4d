//! # Parse Errors
//!
//! Error types for the OpenSCAD parser.
//!
//! ## Example
//!
//! ```rust
//! use openscad_parser::error::ParseError;
//!
//! let error = ParseError::unexpected_token(")", "identifier");
//! println!("{}", error);
//! ```

use crate::span::Span;
use std::fmt;

// =============================================================================
// PARSE ERROR
// =============================================================================

/// A parse error with location information.
///
/// ## Example
///
/// ```rust
/// let error = ParseError::new(
///     ParseErrorKind::UnexpectedToken {
///         found: ")".to_string(),
///         expected: "identifier".to_string(),
///     },
///     Span::from_bytes(5, 6),
/// );
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    /// Error kind with details.
    pub kind: ParseErrorKind,
    /// Source location of error.
    pub span: Span,
}

impl ParseError {
    /// Create a new parse error.
    ///
    /// ## Parameters
    ///
    /// - `kind`: Error kind
    /// - `span`: Source location
    pub const fn new(kind: ParseErrorKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Create unexpected token error.
    ///
    /// ## Parameters
    ///
    /// - `found`: Token that was found
    /// - `expected`: Description of expected token
    pub fn unexpected_token(found: &str, expected: &str) -> Self {
        Self::new(
            ParseErrorKind::UnexpectedToken {
                found: found.to_string(),
                expected: expected.to_string(),
            },
            Span::zero(),
        )
    }

    /// Create unexpected EOF error.
    ///
    /// ## Parameters
    ///
    /// - `expected`: Description of expected token
    pub fn unexpected_eof(expected: &str) -> Self {
        Self::new(
            ParseErrorKind::UnexpectedEof {
                expected: expected.to_string(),
            },
            Span::zero(),
        )
    }

    /// Create error with span.
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at byte {}",
            self.kind, self.span.start.byte
        )
    }
}

impl std::error::Error for ParseError {}

// =============================================================================
// PARSE ERROR KIND
// =============================================================================

/// Kinds of parse errors.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    /// Found unexpected token.
    ///
    /// ## Example
    ///
    /// Expected identifier, found ")"
    UnexpectedToken {
        /// Token that was found.
        found: String,
        /// Description of what was expected.
        expected: String,
    },

    /// Unexpected end of file.
    UnexpectedEof {
        /// Description of what was expected.
        expected: String,
    },

    /// Invalid number literal.
    InvalidNumber {
        /// The invalid text.
        text: String,
    },

    /// Unterminated string literal.
    UnterminatedString,

    /// Invalid escape sequence in string.
    InvalidEscape {
        /// The invalid escape sequence.
        sequence: String,
    },
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedToken { found, expected } => {
                write!(f, "unexpected token '{}', expected {}", found, expected)
            }
            Self::UnexpectedEof { expected } => {
                write!(f, "unexpected end of file, expected {}", expected)
            }
            Self::InvalidNumber { text } => {
                write!(f, "invalid number '{}'", text)
            }
            Self::UnterminatedString => {
                write!(f, "unterminated string literal")
            }
            Self::InvalidEscape { sequence } => {
                write!(f, "invalid escape sequence '{}'", sequence)
            }
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unexpected_token_display() {
        let error = ParseError::unexpected_token(")", "identifier");
        let msg = format!("{}", error);
        assert!(msg.contains("unexpected token ')'"));
        assert!(msg.contains("identifier"));
    }

    #[test]
    fn test_unexpected_eof_display() {
        let error = ParseError::unexpected_eof("semicolon");
        let msg = format!("{}", error);
        assert!(msg.contains("unexpected end of file"));
        assert!(msg.contains("semicolon"));
    }

    #[test]
    fn test_error_with_span() {
        let error = ParseError::unexpected_token("x", "y")
            .with_span(Span::from_bytes(10, 11));
        assert_eq!(error.span.start.byte, 10);
    }
}
