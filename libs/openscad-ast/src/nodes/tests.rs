//! Tests for AST node primitives.

use super::*;

/// Verifies span validation rejects inverted ranges.
///
/// # Examples
/// ```
/// use openscad_ast::Span;
/// assert!(Span::new(5, 1).is_err());
/// ```
#[test]
fn span_validation_fails_for_inverted_ranges() {
    let err = Span::new(5, 1).unwrap_err();
    assert!(matches!(err, SpanError::InvalidRange { .. }));
}

/// Confirms AST node validation enforces positive spans.
///
/// # Examples
/// ```
/// use openscad_ast::{AstNode, Span};
/// let span = Span::new(0, 1).unwrap();
/// let node = AstNode::new("cube", span);
/// assert!(node.validate().is_ok());
/// ```
#[test]
fn node_validation_checks_span_length() {
    let span = Span::new(0, 1).unwrap();
    let node = AstNode::new("cube", span);
    assert!(node.validate().is_ok());
}
