//! Primitive module call parsing (cube, sphere, cylinder).
//!
//! This module handles parsing of OpenSCAD's built-in 3D primitive modules.
//! Each primitive has its own specialized argument parser in the `arguments/`
//! submodule to maintain separation of concerns.
//!
//! # Supported Primitives
//!
//! - **cube**: Rectangular prisms with scalar or vector sizing
//! - **sphere**: Spheres with radius and fragment control
//! - **cylinder**: Cylinders and cones with height and radius parameters
//! - **square**: 2D Rectangles with scalar or vector sizing
//! - **circle**: 2D Circles with radius/diameter and fragment control
//! - **polygon**: 2D Polygons with points and paths
//!
//! # Design Pattern
//!
//! This module uses a **match-based dispatch** pattern:
//! 1. Extract the module name from the CST node
//! 2. Match against known primitives
//! 3. Delegate argument parsing to specialized modules
//! 4. Return `None` for unknown modules (allows extensibility)
//!
//! # Error Handling
//!
//! All primitives require arguments. Missing arguments result in explicit
//! errors with helpful hints about correct usage.
//!
//! # Examples
//!
//! ```
//! use openscad_ast::parse_to_ast;
//!
//! // Cube with scalar size
//! let ast = parse_to_ast("cube(10);").expect("parse succeeds");
//!
//! // Sphere with radius
//! let ast = parse_to_ast("sphere(r=5);").expect("parse succeeds");
//!
//! // Cylinder with height and radius
//! let ast = parse_to_ast("cylinder(h=20, r=5);").expect("parse succeeds");
//! ```

use crate::{ast_types::Statement, Diagnostic, Span};
use tree_sitter::Node;
use super::arguments::cube::parse_cube_arguments;
use super::arguments::sphere::parse_sphere_arguments;
use super::arguments::cylinder::parse_cylinder_arguments;
use super::arguments::square::parse_square_arguments;
use super::arguments::circle::parse_circle_arguments;
use super::arguments::polygon::parse_polygon_arguments;

/// Parses a module call node from the CST into a primitive statement.
///
/// This function identifies the module name and delegates to the appropriate
/// argument parser. It returns `None` for unknown module names, allowing
/// the parser to be extended with new primitives without modifying this code.
///
/// # Arguments
///
/// * `node` - The tree-sitter CST node representing the module call
/// * `source` - The original source code for text extraction
///
/// # Returns
///
/// * `Ok(Some(Statement))` - Successfully parsed primitive
/// * `Ok(None)` - Unknown module name (not a supported primitive)
/// * `Err(Vec<Diagnostic>)` - Parse errors with source locations
///
/// # Supported Modules
///
/// - `cube(size, center)` → [`Statement::Cube`]
/// - `sphere(r, $fn, $fa, $fs)` → [`Statement::Sphere`]
/// - `cylinder(h, r/r1/r2, center, $fn, $fa, $fs)` → [`Statement::Cylinder`]
/// - `square(size, center)` → [`Statement::Square`]
/// - `circle(r|d, $fn, $fa, $fs)` → [`Statement::Circle`]
/// - `polygon(points, paths, convexity)` → [`Statement::Polygon`]
///
/// # Examples
///
/// ```ignore
/// use openscad_ast::parser::module_call::parse_module_call;
///
/// // Typically called by parse_statement, not directly
/// let stmt = parse_module_call(&node, source)?;
/// ```
pub fn parse_module_call(
    node: &Node,
    source: &str,
) -> Result<Option<Statement>, Vec<Diagnostic>> {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    // Extract the module name from the identifier node
    let name_node = children.iter().find(|n| n.kind() == "identifier");
    let name = name_node
        .map(|n| &source[n.byte_range()])
        .unwrap_or("");

    // Dispatch to appropriate primitive parser
    match name {
        "cube" => {
            let args_node = children.iter().find(|n| n.kind() == "arguments");
            if let Some(args) = args_node {
                let (size, center) = parse_cube_arguments(args, source)?;
                let span = Span::new(node.start_byte(), node.end_byte())
                    .map_err(|e| vec![Diagnostic::error(format!("Invalid span: {}", e), Span::new(0, 1).unwrap())])?;

                Ok(Some(Statement::Cube { size, center, span }))
            } else {
                Err(vec![Diagnostic::error(
                    "cube() requires arguments",
                    Span::new(node.start_byte(), node.end_byte()).unwrap(),
                )
                .with_hint("Try: cube(10); or cube([1, 2, 3]);")])
            }
        }
        "sphere" => {
            let args_node = children.iter().find(|n| n.kind() == "arguments");
            if let Some(args) = args_node {
                let (radius, fa, fs, fn_) = parse_sphere_arguments(args, source)?;
                let span = Span::new(node.start_byte(), node.end_byte())
                    .map_err(|e| vec![Diagnostic::error(format!("Invalid span: {}", e), Span::new(0, 1).unwrap())])?;

                Ok(Some(Statement::Sphere { radius, fa, fs, fn_, span }))
            } else {
                Err(vec![Diagnostic::error(
                    "sphere() requires arguments",
                    Span::new(node.start_byte(), node.end_byte()).unwrap(),
                )
                .with_hint("Try: sphere(10); or sphere(r=10);")])
            }
        }
        "cylinder" => {
            let args_node = children.iter().find(|n| n.kind() == "arguments");
            if let Some(args) = args_node {
                let (height, r1, r2, center, fa, fs, fn_) = parse_cylinder_arguments(args, source)?;
                let span = Span::new(node.start_byte(), node.end_byte())
                    .map_err(|e| vec![Diagnostic::error(format!("Invalid span: {}", e), Span::new(0, 1).unwrap())])?;

                Ok(Some(Statement::Cylinder {
                    height,
                    r1,
                    r2,
                    center,
                    fa,
                    fs,
                    fn_,
                    span,
                }))
            } else {
                Err(vec![Diagnostic::error(
                    "cylinder() requires arguments",
                    Span::new(node.start_byte(), node.end_byte()).unwrap(),
                )
                .with_hint("Try: cylinder(h=20, r=5);")])
            }
        }
        "square" => {
            let args_node = children.iter().find(|n| n.kind() == "arguments");
            if let Some(args) = args_node {
                let (size, center) = parse_square_arguments(args, source)?;
                let span = Span::new(node.start_byte(), node.end_byte())
                    .map_err(|e| vec![Diagnostic::error(format!("Invalid span: {}", e), Span::new(0, 1).unwrap())])?;

                Ok(Some(Statement::Square { size, center, span }))
            } else {
                 Err(vec![Diagnostic::error(
                    "square() requires arguments",
                    Span::new(node.start_byte(), node.end_byte()).unwrap(),
                )
                .with_hint("Try: square(10); or square([10, 20]);")])
            }
        }
        "circle" => {
            let args_node = children.iter().find(|n| n.kind() == "arguments");
            if let Some(args) = args_node {
                let (radius, fa, fs, fn_) = parse_circle_arguments(args, source)?;
                let span = Span::new(node.start_byte(), node.end_byte())
                    .map_err(|e| vec![Diagnostic::error(format!("Invalid span: {}", e), Span::new(0, 1).unwrap())])?;

                Ok(Some(Statement::Circle { radius, fa, fs, fn_, span }))
            } else {
                 Err(vec![Diagnostic::error(
                    "circle() requires arguments",
                    Span::new(node.start_byte(), node.end_byte()).unwrap(),
                )
                .with_hint("Try: circle(10); or circle(r=10);")])
            }
        }
        "polygon" => {
            let args_node = children.iter().find(|n| n.kind() == "arguments");
            if let Some(args) = args_node {
                let (points, paths, convexity) = parse_polygon_arguments(args, source)?;
                let span = Span::new(node.start_byte(), node.end_byte())
                    .map_err(|e| vec![Diagnostic::error(format!("Invalid span: {}", e), Span::new(0, 1).unwrap())])?;

                Ok(Some(Statement::Polygon { points, paths, convexity, span }))
            } else {
                 Err(vec![Diagnostic::error(
                    "polygon() requires arguments",
                    Span::new(node.start_byte(), node.end_byte()).unwrap(),
                )
                .with_hint("Try: polygon(points=[[0,0], [10,0], [0,10]]);")])
            }
        }
        // Unknown module - return None to allow extensibility
        _ => Ok(None),
    }
}
