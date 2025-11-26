//! # Geometry IR (Intermediate Representation)
//!
//! The evaluated geometry tree ready for mesh generation.
//! Each node represents a concrete geometry operation with resolved parameters.

use glam::{DMat4, DVec3};
use openscad_ast::Span;

/// A geometry node in the evaluated IR.
///
/// These nodes represent fully resolved geometry operations ready for
/// mesh generation. All parameters are concrete values (no expressions).
#[derive(Debug, Clone, PartialEq)]
pub enum GeometryNode {
    // =========================================================================
    // 3D PRIMITIVES
    // =========================================================================

    /// A cube or rectangular prism.
    Cube {
        size: DVec3,
        center: bool,
        span: Span,
    },

    /// A sphere.
    Sphere {
        radius: f64,
        segments: u32,
        span: Span,
    },

    /// A cylinder or cone.
    Cylinder {
        height: f64,
        radius_bottom: f64,
        radius_top: f64,
        center: bool,
        segments: u32,
        span: Span,
    },

    /// A polyhedron from vertices and faces.
    Polyhedron {
        points: Vec<DVec3>,
        faces: Vec<Vec<u32>>,
        convexity: u32,
        span: Span,
    },

    // =========================================================================
    // 2D PRIMITIVES
    // =========================================================================

    /// A 2D square or rectangle.
    Square {
        size: [f64; 2],
        center: bool,
        span: Span,
    },

    /// A 2D circle.
    Circle {
        radius: f64,
        segments: u32,
        span: Span,
    },

    /// A 2D polygon.
    Polygon {
        points: Vec<[f64; 2]>,
        paths: Option<Vec<Vec<u32>>>,
        span: Span,
    },

    // =========================================================================
    // TRANSFORMATIONS
    // =========================================================================

    /// A transformation applied to children.
    Transform {
        matrix: DMat4,
        children: Vec<GeometryNode>,
        span: Span,
    },

    /// Color applied to children.
    Color {
        color: [f32; 4],
        children: Vec<GeometryNode>,
        span: Span,
    },

    // =========================================================================
    // BOOLEAN OPERATIONS
    // =========================================================================

    /// A boolean operation on children.
    Boolean {
        operation: BooleanOperation,
        children: Vec<GeometryNode>,
        span: Span,
    },

    // =========================================================================
    // EXTRUSIONS
    // =========================================================================

    /// Linear extrusion of 2D children.
    LinearExtrude {
        height: f64,
        center: bool,
        twist: f64,
        slices: u32,
        scale: [f64; 2],
        children: Vec<GeometryNode>,
        span: Span,
    },

    /// Rotational extrusion of 2D children.
    RotateExtrude {
        angle: f64,
        convexity: u32,
        children: Vec<GeometryNode>,
        span: Span,
    },

    // =========================================================================
    // ADVANCED OPERATIONS
    // =========================================================================

    /// Convex hull of children.
    Hull {
        children: Vec<GeometryNode>,
        span: Span,
    },

    /// Minkowski sum of children.
    Minkowski {
        convexity: u32,
        children: Vec<GeometryNode>,
        span: Span,
    },

    /// 2D offset operation.
    Offset {
        amount: OffsetAmount,
        chamfer: bool,
        children: Vec<GeometryNode>,
        span: Span,
    },

    /// Resize operation.
    Resize {
        new_size: DVec3,
        auto_scale: [bool; 3],
        convexity: u32,
        children: Vec<GeometryNode>,
        span: Span,
    },

    /// Empty geometry (no-op).
    Empty {
        span: Span,
    },
}

impl GeometryNode {
    /// Returns the span of this node.
    pub fn span(&self) -> Span {
        match self {
            GeometryNode::Cube { span, .. } => *span,
            GeometryNode::Sphere { span, .. } => *span,
            GeometryNode::Cylinder { span, .. } => *span,
            GeometryNode::Polyhedron { span, .. } => *span,
            GeometryNode::Square { span, .. } => *span,
            GeometryNode::Circle { span, .. } => *span,
            GeometryNode::Polygon { span, .. } => *span,
            GeometryNode::Transform { span, .. } => *span,
            GeometryNode::Color { span, .. } => *span,
            GeometryNode::Boolean { span, .. } => *span,
            GeometryNode::LinearExtrude { span, .. } => *span,
            GeometryNode::RotateExtrude { span, .. } => *span,
            GeometryNode::Hull { span, .. } => *span,
            GeometryNode::Minkowski { span, .. } => *span,
            GeometryNode::Offset { span, .. } => *span,
            GeometryNode::Resize { span, .. } => *span,
            GeometryNode::Empty { span } => *span,
        }
    }

    /// Returns the number of child nodes.
    pub fn child_count(&self) -> usize {
        match self {
            GeometryNode::Cube { .. }
            | GeometryNode::Sphere { .. }
            | GeometryNode::Cylinder { .. }
            | GeometryNode::Polyhedron { .. }
            | GeometryNode::Square { .. }
            | GeometryNode::Circle { .. }
            | GeometryNode::Polygon { .. }
            | GeometryNode::Empty { .. } => 0,
            GeometryNode::Transform { children, .. }
            | GeometryNode::Color { children, .. }
            | GeometryNode::Boolean { children, .. }
            | GeometryNode::LinearExtrude { children, .. }
            | GeometryNode::RotateExtrude { children, .. }
            | GeometryNode::Hull { children, .. }
            | GeometryNode::Minkowski { children, .. }
            | GeometryNode::Offset { children, .. }
            | GeometryNode::Resize { children, .. } => children.len(),
        }
    }

    /// Returns true if this is a 2D primitive.
    pub fn is_2d(&self) -> bool {
        matches!(
            self,
            GeometryNode::Square { .. }
                | GeometryNode::Circle { .. }
                | GeometryNode::Polygon { .. }
        )
    }

    /// Returns true if this is a 3D primitive.
    pub fn is_3d(&self) -> bool {
        matches!(
            self,
            GeometryNode::Cube { .. }
                | GeometryNode::Sphere { .. }
                | GeometryNode::Cylinder { .. }
                | GeometryNode::Polyhedron { .. }
        )
    }
}

/// Boolean operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOperation {
    /// Combine all children into one shape
    Union,
    /// Subtract subsequent children from the first
    Difference,
    /// Keep only the overlapping volume
    Intersection,
}

/// Offset amount type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OffsetAmount {
    /// Round offset with circular arcs at corners
    Radius(f64),
    /// Miter offset with sharp corners
    Delta(f64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_span() {
        let node = GeometryNode::Cube {
            size: DVec3::splat(10.0),
            center: false,
            span: Span::new(0, 10),
        };
        assert_eq!(node.span().start(), 0);
        assert_eq!(node.span().end(), 10);
    }

    #[test]
    fn test_child_count_primitive() {
        let node = GeometryNode::Cube {
            size: DVec3::splat(10.0),
            center: false,
            span: Span::default(),
        };
        assert_eq!(node.child_count(), 0);
    }

    #[test]
    fn test_child_count_transform() {
        let node = GeometryNode::Transform {
            matrix: DMat4::IDENTITY,
            children: vec![
                GeometryNode::Cube {
                    size: DVec3::splat(10.0),
                    center: false,
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        };
        assert_eq!(node.child_count(), 1);
    }

    #[test]
    fn test_is_2d() {
        let square = GeometryNode::Square {
            size: [10.0, 10.0],
            center: false,
            span: Span::default(),
        };
        assert!(square.is_2d());
        assert!(!square.is_3d());
    }

    #[test]
    fn test_is_3d() {
        let cube = GeometryNode::Cube {
            size: DVec3::splat(10.0),
            center: false,
            span: Span::default(),
        };
        assert!(cube.is_3d());
        assert!(!cube.is_2d());
    }

    #[test]
    fn test_boolean_operation() {
        assert_eq!(BooleanOperation::Union, BooleanOperation::Union);
        assert_ne!(BooleanOperation::Union, BooleanOperation::Difference);
    }

    #[test]
    fn test_offset_amount() {
        let radius = OffsetAmount::Radius(5.0);
        let delta = OffsetAmount::Delta(3.0);
        assert_ne!(radius, delta);
    }
}
