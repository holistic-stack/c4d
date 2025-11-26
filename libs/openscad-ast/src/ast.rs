//! # Abstract Syntax Tree
//!
//! Typed AST nodes for OpenSCAD programs. Each node carries source span
//! information for diagnostics and source mapping.
//!
//! ## Node Categories
//!
//! - **Statements**: Top-level program elements (assignments, module calls)
//! - **Expressions**: Values and computations
//! - **Arguments**: Function/module call arguments

use crate::span::Span;
use glam::DVec3;

// =============================================================================
// STATEMENTS
// =============================================================================

/// A top-level statement in an OpenSCAD program.
///
/// Statements are the building blocks of OpenSCAD programs. They include
/// variable assignments, module definitions, and module calls (primitives,
/// transforms, booleans).
///
/// # Example
///
/// ```rust
/// use openscad_ast::{Statement, Span, Expression};
///
/// // A cube primitive call with Expression-based size
/// let stmt = Statement::Cube {
///     size: Expression::Number(10.0),  // Scalar cube
///     center: false,
///     span: Span::new(0, 10),
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    // =========================================================================
    // 3D PRIMITIVES
    // =========================================================================
    
    /// `cube(size, center)` - Creates a cube or rectangular prism.
    ///
    /// # Parameters
    /// - `size`: Expression that evaluates to scalar (cube) or [x, y, z] (rectangular prism)
    /// - `center`: If true, center at origin; if false, corner at origin
    Cube {
        /// Size expression (evaluated at runtime to DVec3)
        size: Expression,
        center: bool,
        span: Span,
    },

    /// `sphere(r|d)` - Creates a sphere.
    ///
    /// # Parameters
    /// - `radius`: Expression that evaluates to the radius
    /// - `segments`: Number of fragments (from $fn or computed from $fa/$fs)
    Sphere {
        /// Radius expression (evaluated at runtime to f64)
        radius: Expression,
        /// Optional local $fn override (if specified in call, e.g., sphere(r=5, $fn=8))
        fn_override: Option<f64>,
        span: Span,
    },

    /// `cylinder(h, r|d, r1|d1, r2|d2, center)` - Creates a cylinder or cone.
    ///
    /// # Parameters
    /// - `height`: Expression for height along Z axis
    /// - `radius_bottom`: Expression for radius at z=0 (or z=-h/2 if centered)
    /// - `radius_top`: Expression for radius at z=h (or z=h/2 if centered)
    /// - `center`: If true, center vertically at origin
    /// - `segments`: Number of fragments around circumference
    Cylinder {
        /// Height expression (evaluated at runtime to f64)
        height: Expression,
        /// Bottom radius expression (evaluated at runtime to f64)
        radius_bottom: Expression,
        /// Top radius expression (evaluated at runtime to f64)
        radius_top: Expression,
        center: bool,
        /// Optional local $fn override (if specified in call, e.g., cylinder(h=10, r=5, $fn=8))
        fn_override: Option<f64>,
        span: Span,
    },

    /// `polyhedron(points, faces, convexity)` - Creates a polyhedron from vertices and faces.
    ///
    /// # Parameters
    /// - `points`: List of 3D vertices
    /// - `faces`: List of face definitions (indices into points)
    /// - `convexity`: Hint for rendering (max intersections along a ray)
    Polyhedron {
        points: Vec<DVec3>,
        faces: Vec<Vec<u32>>,
        convexity: u32,
        span: Span,
    },

    // =========================================================================
    // 2D PRIMITIVES
    // =========================================================================

    /// `square(size, center)` - Creates a 2D square or rectangle.
    Square {
        size: [f64; 2],
        center: bool,
        span: Span,
    },

    /// `circle(r|d)` - Creates a 2D circle.
    Circle {
        radius: f64,
        segments: u32,
        span: Span,
    },

    /// `polygon(points, paths)` - Creates a 2D polygon.
    Polygon {
        points: Vec<[f64; 2]>,
        paths: Option<Vec<Vec<u32>>>,
        span: Span,
    },

    // =========================================================================
    // TRANSFORMATIONS
    // =========================================================================

    /// `translate([x, y, z]) { children }` - Translates children.
    /// Vector is an Expression that evaluates to [x, y, z] at runtime.
    Translate {
        vector: Expression,
        children: Vec<Statement>,
        span: Span,
    },

    /// `rotate([x, y, z]) { children }` or `rotate(a, [vx, vy, vz]) { children }`.
    /// Angles/axis are Expressions evaluated at runtime.
    Rotate {
        /// Euler angles in degrees [rx, ry, rz] or single angle
        angles: Expression,
        /// Optional axis for axis-angle rotation
        axis: Option<Expression>,
        children: Vec<Statement>,
        span: Span,
    },

    /// `scale([x, y, z]) { children }` - Scales children.
    /// Factors is an Expression evaluated at runtime.
    Scale {
        factors: Expression,
        children: Vec<Statement>,
        span: Span,
    },

    /// `mirror([x, y, z]) { children }` - Mirrors children across a plane.
    /// Normal is an Expression evaluated at runtime.
    Mirror {
        /// Normal vector of the mirror plane (expression)
        normal: Expression,
        children: Vec<Statement>,
        span: Span,
    },

    /// `multmatrix(m) { children }` - Applies a 4x4 transformation matrix.
    Multmatrix {
        /// 4x4 transformation matrix in row-major order
        matrix: [[f64; 4]; 4],
        children: Vec<Statement>,
        span: Span,
    },

    /// `resize(newsize, auto) { children }` - Resizes children to fit a bounding box.
    Resize {
        new_size: DVec3,
        auto_scale: [bool; 3],
        children: Vec<Statement>,
        span: Span,
    },

    /// `color("name"|[r,g,b,a]) { children }` - Sets color of children.
    Color {
        /// RGBA color values in range [0, 1]
        color: [f32; 4],
        children: Vec<Statement>,
        span: Span,
    },

    // =========================================================================
    // BOOLEAN OPERATIONS
    // =========================================================================

    /// `union() { children }` - Combines children into one shape.
    Union {
        children: Vec<Statement>,
        span: Span,
    },

    /// `difference() { children }` - Subtracts subsequent children from the first.
    Difference {
        children: Vec<Statement>,
        span: Span,
    },

    /// `intersection() { children }` - Keeps only the overlapping volume.
    Intersection {
        children: Vec<Statement>,
        span: Span,
    },

    // =========================================================================
    // EXTRUSIONS
    // =========================================================================

    /// `linear_extrude(height, center, twist, slices, scale) { 2D children }`.
    LinearExtrude {
        height: f64,
        center: bool,
        twist: f64,
        slices: u32,
        scale: [f64; 2],
        children: Vec<Statement>,
        span: Span,
    },

    /// `rotate_extrude(angle, convexity) { 2D children }`.
    RotateExtrude {
        angle: f64,
        convexity: u32,
        children: Vec<Statement>,
        span: Span,
    },

    // =========================================================================
    // ADVANCED OPERATIONS
    // =========================================================================

    /// `hull() { children }` - Computes convex hull of children.
    Hull {
        children: Vec<Statement>,
        span: Span,
    },

    /// `minkowski(convexity) { children }` - Computes Minkowski sum.
    Minkowski {
        convexity: u32,
        children: Vec<Statement>,
        span: Span,
    },

    /// `offset(r|delta, chamfer) { 2D children }` - Offsets 2D shapes.
    Offset {
        amount: OffsetAmount,
        chamfer: bool,
        children: Vec<Statement>,
        span: Span,
    },

    // =========================================================================
    // VARIABLES & CONTROL FLOW
    // =========================================================================

    /// Variable assignment: `name = value;` or `$fn = 32;`.
    Assignment {
        name: String,
        value: Expression,
        span: Span,
    },

    /// Module definition: `module name(params) { body }`.
    ModuleDefinition {
        name: String,
        parameters: Vec<Parameter>,
        body: Vec<Statement>,
        span: Span,
    },

    /// Function definition: `function name(params) = expression;`.
    /// Functions return a value computed from an expression.
    FunctionDefinition {
        name: String,
        parameters: Vec<Parameter>,
        /// The expression that computes the return value
        body: Expression,
        span: Span,
    },

    /// Module call: `name(args) { children }` or `name(args);`.
    ModuleCall {
        name: String,
        arguments: Vec<Argument>,
        children: Vec<Statement>,
        span: Span,
    },

    /// For loop: `for (var = range) { body }`.
    ForLoop {
        variable: String,
        range: Expression,
        body: Vec<Statement>,
        span: Span,
    },

    /// If statement: `if (condition) { then } else { else }`.
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
        span: Span,
    },

    /// Echo statement: `echo(args);`.
    Echo {
        arguments: Vec<Expression>,
        span: Span,
    },

    /// A statement with a debug modifier (*, !, #, %).
    /// 
    /// # Example
    /// 
    /// ```openscad
    /// *cube(10);           // Disable - not rendered
    /// !cube(10);           // ShowOnly - only this is rendered
    /// #cube(10);           // Highlight - rendered in magenta
    /// %cube(10);           // Transparent - rendered semi-transparent
    /// ```
    Modified {
        /// The modifier applied to this statement
        modifier: Modifier,
        /// The statement being modified
        child: Box<Statement>,
        span: Span,
    },
}

impl Statement {
    /// Returns the source span of this statement.
    pub fn span(&self) -> Span {
        match self {
            Statement::Cube { span, .. } => *span,
            Statement::Sphere { span, .. } => *span,
            Statement::Cylinder { span, .. } => *span,
            Statement::Polyhedron { span, .. } => *span,
            Statement::Square { span, .. } => *span,
            Statement::Circle { span, .. } => *span,
            Statement::Polygon { span, .. } => *span,
            Statement::Translate { span, .. } => *span,
            Statement::Rotate { span, .. } => *span,
            Statement::Scale { span, .. } => *span,
            Statement::Mirror { span, .. } => *span,
            Statement::Multmatrix { span, .. } => *span,
            Statement::Resize { span, .. } => *span,
            Statement::Color { span, .. } => *span,
            Statement::Union { span, .. } => *span,
            Statement::Difference { span, .. } => *span,
            Statement::Intersection { span, .. } => *span,
            Statement::LinearExtrude { span, .. } => *span,
            Statement::RotateExtrude { span, .. } => *span,
            Statement::Hull { span, .. } => *span,
            Statement::Minkowski { span, .. } => *span,
            Statement::Offset { span, .. } => *span,
            Statement::Assignment { span, .. } => *span,
            Statement::ModuleDefinition { span, .. } => *span,
            Statement::FunctionDefinition { span, .. } => *span,
            Statement::ModuleCall { span, .. } => *span,
            Statement::ForLoop { span, .. } => *span,
            Statement::If { span, .. } => *span,
            Statement::Echo { span, .. } => *span,
            Statement::Modified { span, .. } => *span,
        }
    }
}

// =============================================================================
// OFFSET AMOUNT
// =============================================================================

/// The amount for an offset operation.
///
/// OpenSCAD supports two types of offset:
/// - `r`: Round offset (circular arcs at corners)
/// - `delta`: Miter offset (sharp corners)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OffsetAmount {
    /// Round offset with circular arcs at corners
    Radius(f64),
    /// Miter offset with sharp corners
    Delta(f64),
}

// =============================================================================
// MODIFIERS
// =============================================================================

/// OpenSCAD debug modifiers that affect rendering behavior.
///
/// Modifiers are prefix operators placed before a statement to change
/// how that object is rendered in preview mode.
///
/// # OpenSCAD Modifier Reference
///
/// | Modifier | Name | Description |
/// |----------|------|-------------|
/// | `*` | Disable | Object is not rendered at all |
/// | `!` | ShowOnly | Only this object is rendered (root modifier) |
/// | `#` | Highlight | Object is rendered in magenta |
/// | `%` | Transparent | Object is rendered semi-transparent |
///
/// # Example
///
/// ```rust
/// use openscad_ast::Modifier;
///
/// let disable = Modifier::Disable;     // *cube(10);
/// let show_only = Modifier::ShowOnly;  // !cube(10);
/// let highlight = Modifier::Highlight; // #cube(10);
/// let transparent = Modifier::Transparent; // %cube(10);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    /// `*` - Disables the object (not rendered in preview or export)
    Disable,
    /// `!` - Shows only this object (root modifier for debugging)
    ShowOnly,
    /// `#` - Highlights the object in magenta for debugging
    Highlight,
    /// `%` - Renders the object as semi-transparent
    Transparent,
}

impl Modifier {
    /// Parses a modifier from a character.
    ///
    /// # Arguments
    ///
    /// * `c` - The modifier character
    ///
    /// # Returns
    ///
    /// The corresponding modifier, or None if not a valid modifier character.
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '*' => Some(Self::Disable),
            '!' => Some(Self::ShowOnly),
            '#' => Some(Self::Highlight),
            '%' => Some(Self::Transparent),
            _ => None,
        }
    }

    /// Returns the character representation of this modifier.
    pub fn as_char(&self) -> char {
        match self {
            Self::Disable => '*',
            Self::ShowOnly => '!',
            Self::Highlight => '#',
            Self::Transparent => '%',
        }
    }
}

// =============================================================================
// EXPRESSIONS
// =============================================================================

/// An expression that evaluates to a value.
///
/// Expressions are used in assignments, conditions, and arguments.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// A numeric literal: `42`, `3.14`
    Number(f64),

    /// A boolean literal: `true`, `false`
    Boolean(bool),

    /// A string literal: `"hello"`
    String(String),

    /// A vector literal: `[1, 2, 3]`
    Vector(Vec<Expression>),

    /// A range: `[start:end]` or `[start:step:end]`
    Range {
        start: Box<Expression>,
        step: Option<Box<Expression>>,
        end: Box<Expression>,
    },

    /// A variable reference: `x`, `$fn`
    Variable(String),

    /// A unary operation: `-x`, `!x`
    Unary {
        operator: UnaryOp,
        operand: Box<Expression>,
    },

    /// A binary operation: `a + b`, `a && b`
    Binary {
        left: Box<Expression>,
        operator: BinaryOp,
        right: Box<Expression>,
    },

    /// A ternary conditional: `cond ? then : else`
    Ternary {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },

    /// A function call: `sin(x)`, `len(v)`
    FunctionCall {
        name: String,
        arguments: Vec<Argument>,
    },

    /// Array indexing: `arr[i]`
    Index {
        array: Box<Expression>,
        index: Box<Expression>,
    },

    /// Undefined value (used for optional parameters)
    Undef,
}

// =============================================================================
// OPERATORS
// =============================================================================

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    /// Negation: `-x`
    Negate,
    /// Logical not: `!x`
    Not,
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,

    // Comparison
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Logical
    And,
    Or,
}

// =============================================================================
// ARGUMENTS & PARAMETERS
// =============================================================================

/// An argument in a function or module call.
///
/// Arguments can be positional or named.
///
/// # Example
///
/// ```rust
/// use openscad_ast::{Argument, Expression};
///
/// // Positional argument: cube(10)
/// let pos = Argument::positional(Expression::Number(10.0));
///
/// // Named argument: cube(size=10)
/// let named = Argument::named("size", Expression::Number(10.0));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    /// Optional parameter name (None for positional arguments)
    pub name: Option<String>,
    /// The argument value
    pub value: Expression,
}

impl Argument {
    /// Creates a positional argument.
    pub fn positional(value: Expression) -> Self {
        Self { name: None, value }
    }

    /// Creates a named argument.
    pub fn named(name: impl Into<String>, value: Expression) -> Self {
        Self {
            name: Some(name.into()),
            value,
        }
    }
}

/// A parameter in a module or function definition.
///
/// Parameters can have default values.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Optional default value
    pub default: Option<Expression>,
}

impl Parameter {
    /// Creates a required parameter (no default).
    pub fn required(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default: None,
        }
    }

    /// Creates an optional parameter with a default value.
    pub fn optional(name: impl Into<String>, default: Expression) -> Self {
        Self {
            name: name.into(),
            default: Some(default),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_statement_span() {
        let stmt = Statement::Cube {
            size: Expression::Number(10.0),
            center: false,
            span: Span::new(0, 10),
        };
        assert_eq!(stmt.span().start(), 0);
        assert_eq!(stmt.span().end(), 10);
    }

    #[test]
    fn test_argument_positional() {
        let arg = Argument::positional(Expression::Number(42.0));
        assert!(arg.name.is_none());
        assert_eq!(arg.value, Expression::Number(42.0));
    }

    #[test]
    fn test_argument_named() {
        let arg = Argument::named("size", Expression::Number(10.0));
        assert_eq!(arg.name, Some("size".to_string()));
    }

    #[test]
    fn test_parameter_required() {
        let param = Parameter::required("x");
        assert_eq!(param.name, "x");
        assert!(param.default.is_none());
    }

    #[test]
    fn test_parameter_optional() {
        let param = Parameter::optional("x", Expression::Number(0.0));
        assert_eq!(param.name, "x");
        assert!(param.default.is_some());
    }

    #[test]
    fn test_offset_amount_variants() {
        let radius = OffsetAmount::Radius(5.0);
        let delta = OffsetAmount::Delta(3.0);
        
        match radius {
            OffsetAmount::Radius(r) => assert_eq!(r, 5.0),
            _ => panic!("Expected Radius"),
        }
        
        match delta {
            OffsetAmount::Delta(d) => assert_eq!(d, 3.0),
            _ => panic!("Expected Delta"),
        }
    }
}
