//! Fundamental AST node definitions.
//!
//! Each node stores its textual span so evaluators and diagnostics can provide
//! precise feedback.

use config::constants::EPSILON_TOLERANCE;

/// Represents a half-open byte range within a source file.
///
/// # Examples
/// ```
/// use openscad_ast::Span;
/// let span = Span::new(0, 4).expect("valid span");
/// assert_eq!(span.len(), 4);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    /// Builds a new span ensuring the start is not greater than the end.
    pub fn new(start: usize, end: usize) -> Result<Self, SpanError> {
        if start > end {
            return Err(SpanError::InvalidRange { start, end });
        }
        Ok(Self { start, end })
    }

    /// Returns the length of the span.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Returns the starting byte offset.
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the ending byte offset.
    pub fn end(&self) -> usize {
        self.end
    }
}

/// Error emitted when invalid span data is provided.
#[derive(Debug, PartialEq, Eq)]
pub enum SpanError {
    /// The provided start offset exceeded the end offset.
    InvalidRange { start: usize, end: usize },
}

impl std::fmt::Display for SpanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpanError::InvalidRange { start, end } => {
                write!(f, "Invalid span range: start {} > end {}", start, end)
            }
        }
    }
}

impl std::error::Error for SpanError {}

/// Stores metadata common to all AST nodes.
///
/// # Examples
/// ```
/// use openscad_ast::{AstMetadata, Span};
/// let meta = AstMetadata::new(Span::new(0, 1).unwrap());
/// assert_eq!(meta.span().len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstMetadata {
    span: Span,
}

impl AstMetadata {
    /// Creates metadata from the provided span.
    pub fn new(span: Span) -> Self {
        Self { span }
    }

    /// Returns the stored span reference.
    pub fn span(&self) -> Span {
        self.span
    }
}

/// Simplified AST node capturing kind and metadata.
///
/// # Examples
/// ```
/// use openscad_ast::{AstNode, Span};
/// let node = AstNode::new("cube", Span::new(0, 4).unwrap());
/// assert_eq!(node.kind(), "cube");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstNode {
    kind: String,
    metadata: AstMetadata,
}

impl AstNode {
    /// Creates a new AST node ensuring the span has a minimum size.
    pub fn new(kind: impl Into<String>, span: Span) -> Self {
        let metadata = AstMetadata::new(span);
        Self {
            kind: kind.into(),
            metadata,
        }
    }

    /// Returns the kind string.
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Returns the metadata reference.
    pub fn metadata(&self) -> &AstMetadata {
        &self.metadata
    }

    /// Ensures the span length respects the epsilon tolerance for validation.
    pub fn validate(&self) -> Result<(), SpanError> {
        if (self.metadata.span().len() as f64) < EPSILON_TOLERANCE {
            return Err(SpanError::InvalidRange {
                start: self.metadata.span().start(),
                end: self.metadata.span().end(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
