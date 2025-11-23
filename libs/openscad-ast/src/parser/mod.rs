//! Modular OpenSCAD parser orchestrating CST → AST conversion.
//!
//! # Architecture Overview
//!
//! This parser follows the Single Responsibility Principle (SRP) by splitting
//! parsing logic into focused modules:
//!
//! - **`statement.rs`**: Top-level statement dispatcher
//! - **`module_call.rs`**: Primitive module parsing (cube, sphere, cylinder)
//! - **`transform_chain.rs`**: Transform operations (translate, rotate, scale)
//! - **`assignments.rs`**: Variable declaration parsing
//! - **`arguments/`**: Argument parsing for each primitive type
//!   - `cube.rs`: Cube-specific argument handling
//!   - `sphere.rs`: Sphere-specific argument handling
//!   - `cylinder.rs`: Cylinder-specific argument handling
//!   - `shared.rs`: Common parsing utilities (f64, u32, bool, vectors)
//!
//! # Design Principles
//!
//! - **SRP**: Each module has a single, well-defined responsibility
//! - **DRY**: Shared utilities are centralized in `arguments/shared.rs`
//! - **TDD**: All modules include comprehensive tests
//! - **File Size**: All files kept under 500 lines
//! - **Error Handling**: Explicit errors, no fallback mechanisms
//!
//! # Usage
//!
//! The main entry point is [`parse_to_ast`], which:
//! 1. Parses OpenSCAD source to a Concrete Syntax Tree (CST) using tree-sitter
//! 2. Converts the CST to a typed Abstract Syntax Tree (AST)
//! 3. Returns structured diagnostics for any errors
//!
//! # Examples
//!
//! ```
//! use openscad_ast::parse_to_ast;
//!
//! // Parse a simple cube
//! let ast = parse_to_ast("cube(10);").expect("parse succeeds");
//! assert_eq!(ast.len(), 1);
//!
//! // Parse with transforms
//! let ast = parse_to_ast("translate([1,0,0]) cube([2,3,4]);").expect("parse succeeds");
//! assert_eq!(ast.len(), 1);
//! ```
//!
//! # Extension Guide
//!
//! To add support for a new primitive:
//! 1. Create `arguments/newprimitive.rs` with argument parsing
//! 2. Add the primitive case to `module_call.rs`
//! 3. Add corresponding AST variant to `ast_types.rs`
//! 4. Write comprehensive tests in the new module
//!
//! For more details, see `specs/split-parser/plan.md`.

use crate::{ast_types::*, Diagnostic, Span};
use tree_sitter_openscad_parser::parse_source as parse_cst;
use crate::parser::statement::parse_statement;

mod assignments;
pub(crate) mod arguments;
mod module_call;
mod statement;
mod transform_chain;

#[cfg(test)]
pub mod tests;

/// Parses OpenSCAD source code into a typed Abstract Syntax Tree (AST).
///
/// This is the main entry point for the parser. It performs a two-stage parse:
/// 1. **CST Generation**: Uses tree-sitter to create a Concrete Syntax Tree
/// 2. **AST Conversion**: Transforms the CST into strongly-typed AST nodes
///
/// # Arguments
///
/// * `source` - The OpenSCAD source code to parse
///
/// # Returns
///
/// * `Ok(Vec<Statement>)` - Successfully parsed AST statements
/// * `Err(Vec<Diagnostic>)` - Parse errors with source locations and hints
///
/// # Error Handling
///
/// This function returns explicit errors for:
/// - Tree-sitter parse failures
/// - Syntax errors in the source code
/// - Invalid argument types or values
/// - Unknown module names
///
/// All errors include source spans and helpful hints for debugging.
///
/// # Examples
///
/// ```
/// use openscad_ast::parse_to_ast;
///
/// // Parse a simple cube
/// let ast = parse_to_ast("cube(10);").expect("parse succeeds");
/// assert_eq!(ast.len(), 1);
///
/// // Parse multiple statements
/// let ast = parse_to_ast("cube(5); sphere(3);").expect("parse succeeds");
/// assert_eq!(ast.len(), 2);
///
/// // Parse with transforms
/// let ast = parse_to_ast("translate([1,0,0]) rotate([0,90,0]) cube(10);")
///     .expect("parse succeeds");
/// assert_eq!(ast.len(), 1);
///
/// // Handle errors
/// let result = parse_to_ast("cube(");
/// assert!(result.is_err());
/// ```
pub fn parse_to_ast(source: &str) -> Result<Vec<Statement>, Vec<Diagnostic>> {
    // Parse source to Concrete Syntax Tree using tree-sitter
    let tree = parse_cst(source).map_err(|e| {
        vec![Diagnostic::error(
            format!("Parse error: {}", e),
            Span::new(0, source.len()).unwrap_or_else(|_| Span::new(0, 1).unwrap()),
        )]
    })?;

    let root = tree.root_node();
    
    // Check for syntax errors in the CST
    if root.has_error() {
        return Err(vec![Diagnostic::error(
            "Syntax error in source code",
            Span::new(0, source.len()).unwrap_or_else(|_| Span::new(0, 1).unwrap()),
        )
        .with_hint("Check for missing semicolons or parentheses")]);
    }

    // Convert CST nodes to typed AST statements
    let mut statements = Vec::new();
    let mut cursor = root.walk();
    
    for child in root.children(&mut cursor) {
        if let Some(stmt) = parse_statement(&child, source)? {
            statements.push(stmt);
        }
    }

    Ok(statements)
}
