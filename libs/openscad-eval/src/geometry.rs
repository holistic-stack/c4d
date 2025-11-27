//! # Geometry Types
//!
//! Evaluated geometry node types representing resolved OpenSCAD geometry.
//!
//! These types have all expressions evaluated - sizes are concrete numbers,
//! transforms are resolved matrices, etc.

use serde::{Deserialize, Serialize};

// =============================================================================
// EVALUATED AST
// =============================================================================

/// Result of AST evaluation.
///
/// Contains the root geometry node and any warnings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatedAst {
    /// Root geometry node.
    pub geometry: GeometryNode,
    /// Evaluation warnings.
    pub warnings: Vec<String>,
}

impl EvaluatedAst {
    /// Create new evaluated AST.
    pub fn new(geometry: GeometryNode) -> Self {
        Self {
            geometry,
            warnings: Vec::new(),
        }
    }

    /// Create with warnings.
    pub fn with_warnings(geometry: GeometryNode, warnings: Vec<String>) -> Self {
        Self { geometry, warnings }
    }
}

// =============================================================================
// GEOMETRY NODE
// =============================================================================

/// A node in the evaluated geometry tree.
///
/// All values are fully resolved (no variables, expressions evaluated).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeometryNode {
    // =========================================================================
    // 3D PRIMITIVES
    // =========================================================================

    /// Cube primitive.
    ///
    /// ## OpenSCAD Equivalent
    /// 
    /// ```text
    /// cube(size);
    /// cube([x, y, z], center=true);
    /// ```
    Cube {
        /// Size as [x, y, z].
        size: [f64; 3],
        /// Whether centered at origin.
        center: bool,
    },

    /// Sphere primitive.
    ///
    /// ## OpenSCAD Equivalent
    ///
    /// ```text
    /// sphere(r=5);
    /// sphere(d=10, $fn=32);
    /// ```
    Sphere {
        /// Radius.
        radius: f64,
        /// Number of fragments ($fn).
        fn_: u32,
    },

    /// Cylinder primitive.
    ///
    /// ## OpenSCAD Equivalent
    ///
    /// ```text
    /// cylinder(h=10, r=5);
    /// cylinder(h=10, r1=5, r2=3, center=true);
    /// ```
    Cylinder {
        /// Height.
        height: f64,
        /// Bottom radius.
        radius1: f64,
        /// Top radius.
        radius2: f64,
        /// Whether centered.
        center: bool,
        /// Number of fragments.
        fn_: u32,
    },

    /// Polyhedron primitive.
    Polyhedron {
        /// Vertex positions.
        points: Vec<[f64; 3]>,
        /// Face indices.
        faces: Vec<Vec<usize>>,
    },

    // =========================================================================
    // 2D PRIMITIVES
    // =========================================================================

    /// Circle primitive.
    Circle {
        /// Radius.
        radius: f64,
        /// Number of fragments.
        fn_: u32,
    },

    /// Square/rectangle primitive.
    Square {
        /// Size as [x, y].
        size: [f64; 2],
        /// Whether centered.
        center: bool,
    },

    /// Polygon primitive.
    Polygon {
        /// Vertex positions.
        points: Vec<[f64; 2]>,
        /// Optional paths.
        paths: Option<Vec<Vec<usize>>>,
    },

    // =========================================================================
    // TRANSFORMS
    // =========================================================================

    /// Translation transform.
    Translate {
        /// Translation vector [x, y, z].
        offset: [f64; 3],
        /// Child geometry.
        child: Box<GeometryNode>,
    },

    /// Rotation transform.
    Rotate {
        /// Rotation angles [x, y, z] in degrees.
        angles: [f64; 3],
        /// Child geometry.
        child: Box<GeometryNode>,
    },

    /// Scale transform.
    Scale {
        /// Scale factors [x, y, z].
        factors: [f64; 3],
        /// Child geometry.
        child: Box<GeometryNode>,
    },

    /// Mirror transform.
    Mirror {
        /// Mirror plane normal.
        normal: [f64; 3],
        /// Child geometry.
        child: Box<GeometryNode>,
    },

    /// General matrix transform.
    Multmatrix {
        /// 4x4 transformation matrix.
        matrix: [[f64; 4]; 4],
        /// Child geometry.
        child: Box<GeometryNode>,
    },

    /// Color modifier.
    Color {
        /// RGBA color.
        rgba: [f64; 4],
        /// Child geometry.
        child: Box<GeometryNode>,
    },

    // =========================================================================
    // BOOLEAN OPERATIONS
    // =========================================================================

    /// Union of children.
    Union {
        /// Child geometries.
        children: Vec<GeometryNode>,
    },

    /// Difference (first child minus rest).
    Difference {
        /// Child geometries.
        children: Vec<GeometryNode>,
    },

    /// Intersection of children.
    Intersection {
        /// Child geometries.
        children: Vec<GeometryNode>,
    },

    // =========================================================================
    // EXTRUSIONS
    // =========================================================================

    /// Linear extrusion.
    LinearExtrude {
        /// Extrusion height.
        height: f64,
        /// Twist angle in degrees.
        twist: f64,
        /// Scale at top.
        scale: [f64; 2],
        /// Number of slices.
        slices: u32,
        /// Whether centered.
        center: bool,
        /// Child 2D geometry.
        child: Box<GeometryNode>,
    },

    /// Rotational extrusion.
    RotateExtrude {
        /// Sweep angle in degrees.
        angle: f64,
        /// Number of fragments.
        fn_: u32,
        /// Child 2D geometry.
        child: Box<GeometryNode>,
    },

    // =========================================================================
    // META
    // =========================================================================

    /// Group of geometries (implicit union).
    Group {
        /// Child geometries.
        children: Vec<GeometryNode>,
    },

    /// Empty geometry (for conditionals that produce nothing).
    Empty,
}

impl GeometryNode {
    /// Check if this is an empty node.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Check if this is a 2D node.
    pub fn is_2d(&self) -> bool {
        matches!(self, Self::Circle { .. } | Self::Square { .. } | Self::Polygon { .. })
    }

    /// Check if this is a 3D node.
    pub fn is_3d(&self) -> bool {
        matches!(
            self,
            Self::Cube { .. }
                | Self::Sphere { .. }
                | Self::Cylinder { .. }
                | Self::Polyhedron { .. }
        )
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_node() {
        let cube = GeometryNode::Cube {
            size: [10.0, 10.0, 10.0],
            center: false,
        };
        assert!(cube.is_3d());
        assert!(!cube.is_2d());
    }

    #[test]
    fn test_circle_node() {
        let circle = GeometryNode::Circle {
            radius: 5.0,
            fn_: 32,
        };
        assert!(circle.is_2d());
        assert!(!circle.is_3d());
    }

    #[test]
    fn test_empty_node() {
        let empty = GeometryNode::Empty;
        assert!(empty.is_empty());
    }
}
