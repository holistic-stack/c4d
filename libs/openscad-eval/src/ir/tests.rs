//! Tests for the geometry IR nodes.

use super::*;
use glam::DVec3;

use openscad_ast::Span;

/// Valid cube creation scenario.
///
/// # Examples
/// ```
/// use glam::DVec3;
/// use openscad_eval::ir::GeometryNode;
/// use openscad_ast::Span;
/// let span = Span::new(0, 10).unwrap();
/// assert!(GeometryNode::cube(DVec3::splat(1.0), false, span).is_ok());
/// ```
#[test]
fn cube_creation_succeeds() {
    let size = DVec3::new(1.0, 2.0, 3.0);
    let span = Span::new(0, 10).unwrap();
    let node = GeometryNode::cube(size, false, span).expect("valid cube");
    assert_eq!(node.size(), size);
}

/// Ensures validation rejects small values.
///
/// # Examples
/// ```
/// use glam::DVec3;
/// use openscad_eval::ir::GeometryNode;
/// use openscad_ast::Span;
/// let span = Span::new(0, 10).unwrap();
/// assert!(GeometryNode::cube(DVec3::ZERO, false, span).is_err());
/// ```
#[test]
fn cube_creation_fails_when_too_small() {
    let span = Span::new(0, 10).unwrap();
    let err = GeometryNode::cube(DVec3::ZERO, false, span).unwrap_err();
    assert!(matches!(
        err,
        GeometryValidationError::CubeSizeTooSmall { .. }
    ));
}
