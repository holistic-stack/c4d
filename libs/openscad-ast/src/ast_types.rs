/// AST node types for OpenSCAD.
///
/// This module defines strongly-typed AST nodes that represent
/// OpenSCAD language constructs.

use crate::Span;

/// A statement in OpenSCAD source code.
///
/// # Examples
/// ```
/// use openscad_ast::{Statement, CubeSize, Span};
///
/// let span = Span::new(0, 10).unwrap();
/// let stmt = Statement::Cube {
///     size: CubeSize::Scalar(10.0),
///     center: None,
///     span,
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// A cube primitive.
    ///
    /// Examples:
    /// - `cube(10);` → size=Scalar(10), center=None
    /// - `cube([1, 2, 3]);` → size=Vector([1,2,3]), center=None
    /// - `cube(10, center=true);` → size=Scalar(10), center=Some(true)
    Cube {
        /// The size of the cube (scalar or vector).
        size: CubeSize,
        /// Whether the cube should be centered at the origin.
        /// None means use the default (false in OpenSCAD).
        center: Option<bool>,
        /// Source span for error reporting.
        span: Span,
    },
    /// A sphere primitive.
    ///
    /// Examples:
    /// - `sphere(10);` → radius=10, fa/fs/fn=None
    /// - `sphere(r=10, $fn=100);` → radius=10
    Sphere {
        /// The radius of the sphere.
        radius: f64,
        /// The resolution parameter $fa (minimum angle).
        fa: Option<f64>,
        /// The resolution parameter $fs (minimum size).
        fs: Option<f64>,
        /// The resolution parameter $fn (number of fragments).
        fn_: Option<u32>,
        /// Source span for error reporting.
        span: Span,
    },
    /// A cylinder (or cone) primitive.
    ///
    /// Examples:
    /// - `cylinder(h=20, r=5);` → height=20, r1=r2=5
    /// - `cylinder(h=15, r1=5, r2=2, center=true, $fn=64);`
    Cylinder {
        /// Height of the cylinder along Z.
        height: f64,
        /// Bottom radius (`r1` in OpenSCAD).
        r1: f64,
        /// Top radius (`r2` in OpenSCAD).
        r2: f64,
        /// Whether the cylinder is centered around the origin.
        center: bool,
        /// Resolution overrides.
        fa: Option<f64>,
        /// Resolution overrides.
        fs: Option<f64>,
        /// Resolution overrides.
        fn_: Option<u32>,
        /// Source span for diagnostics.
        span: Span,
    },
    /// A variable assignment.
    ///
    /// Examples:
    /// - `$fn = 50;`
    /// - `x = 10;`
    Assignment {
        /// The name of the variable being assigned.
        name: String,
        /// The value being assigned.
        /// For Task 3.2 we only support scalar/numeric assignments.
        value: f64,
        /// Source span for error reporting.
        span: Span,
    },
    /// A transformation (translate, rotate, scale).
    Translate {
        /// Translation vector.
        vector: [f64; 3],
        /// The statement being transformed.
        child: Box<Statement>,
        /// Source span.
        span: Span,
    },
    /// A rotation transformation.
    Rotate {
        /// Rotation vector (degrees).
        vector: [f64; 3],
        /// The statement being transformed.
        child: Box<Statement>,
        /// Source span.
        span: Span,
    },
    /// A scaling transformation.
    Scale {
        /// Scale vector.
        vector: [f64; 3],
        /// The statement being transformed.
        child: Box<Statement>,
        /// Source span.
        span: Span,
    },
}

/// Size specification for a cube.
///
/// # Examples
/// ```
/// use openscad_ast::CubeSize;
///
/// let scalar = CubeSize::Scalar(10.0);
/// let vector = CubeSize::Vector([1.0, 2.0, 3.0]);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum CubeSize {
    /// A scalar size (applies to all dimensions).
    ///
    /// Example: `cube(10);` → `CubeSize::Scalar(10.0)`
    Scalar(f64),
    
    /// A vector size (x, y, z).
    ///
    /// Example: `cube([1, 2, 3]);` → `CubeSize::Vector([1.0, 2.0, 3.0])`
    Vector([f64; 3]),
}

impl CubeSize {
    /// Converts the cube size to a 3D vector.
    ///
    /// For scalar sizes, the value is replicated across all dimensions.
    ///
    /// # Examples
    /// ```
    /// use openscad_ast::CubeSize;
    ///
    /// let scalar = CubeSize::Scalar(10.0);
    /// assert_eq!(scalar.to_vec3(), [10.0, 10.0, 10.0]);
    ///
    /// let vector = CubeSize::Vector([1.0, 2.0, 3.0]);
    /// assert_eq!(vector.to_vec3(), [1.0, 2.0, 3.0]);
    /// ```
    pub fn to_vec3(&self) -> [f64; 3] {
        match self {
            CubeSize::Scalar(s) => [*s, *s, *s],
            CubeSize::Vector(v) => *v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_size_scalar_to_vec3() {
        let size = CubeSize::Scalar(10.0);
        assert_eq!(size.to_vec3(), [10.0, 10.0, 10.0]);
    }

    #[test]
    fn test_cube_size_vector_to_vec3() {
        let size = CubeSize::Vector([1.0, 2.0, 3.0]);
        assert_eq!(size.to_vec3(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_statement_creation() {
        let span = Span::new(0, 10).unwrap();
        let stmt = Statement::Cube {
            size: CubeSize::Scalar(10.0),
            center: None,
            span,
        };
        
        match stmt {
            Statement::Cube { size, center, .. } => {
                assert_eq!(size, CubeSize::Scalar(10.0));
                assert_eq!(center, None);
            }
            _ => panic!("Expected Cube"),
        }
    }
}
