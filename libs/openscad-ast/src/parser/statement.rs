//! Top-level statement parsing and dispatching.
//!
//! This module acts as the primary dispatcher for parsing OpenSCAD statements
//! from the Concrete Syntax Tree (CST). It routes different node types to their
//! specialized parsers while maintaining a clean separation of concerns.
//!
//! # Responsibilities
//!
//! - Identify statement types from CST nodes
//! - Dispatch to appropriate specialized parsers:
//!   - `module_call` → [`module_call::parse_module_call`]
//!   - `transform_chain` → [`transform_chain::parse_transform_chain`]
//!   - `var_declaration` → [`assignments::parse_var_declaration`]
//! - Ignore non-statement nodes (comments, whitespace)
//!
//! # Design Pattern
//!
//! This module implements the **Strategy Pattern** by delegating parsing
//! to specialized modules based on the CST node kind. This keeps the code
//! modular and makes it easy to add new statement types.
//!
//! # Examples
//!
//! ```
//! use openscad_ast::parse_to_ast;
//!
//! // Module calls are dispatched to module_call parser
//! let ast = parse_to_ast("cube(10);").expect("parse succeeds");
//!
//! // Transform chains are dispatched to transform_chain parser
//! let ast = parse_to_ast("translate([1,0,0]) cube(5);").expect("parse succeeds");
//!
//! // Variable declarations are dispatched to assignments parser
//! let ast = parse_to_ast("$fn = 50;").expect("parse succeeds");
//! ```

use crate::{ast_types::Statement, Diagnostic};
use tree_sitter::Node;
use super::module_call::parse_module_call;
use super::transform_chain::parse_transform_chain;
use super::assignments::parse_var_declaration;

/// Parses a statement node from the CST and dispatches to specialized parsers.
///
/// This function examines the CST node kind and routes it to the appropriate
/// parser module. It returns `None` for non-statement nodes (like comments)
/// which should be ignored during AST construction.
///
/// # Arguments
///
/// * `node` - The tree-sitter CST node to parse
/// * `source` - The original source code (for extracting text ranges)
///
/// # Returns
///
/// * `Ok(Some(Statement))` - Successfully parsed statement
/// * `Ok(None)` - Non-statement node (should be ignored)
/// * `Err(Vec<Diagnostic>)` - Parse errors with source locations
///
/// # Supported Node Types
///
/// - `module_call`: Direct primitive calls (e.g., `cube(10);`)
/// - `transform_chain`: Transforms with children (e.g., `translate([1,0,0]) cube(5);`)
/// - `var_declaration`: Variable assignments (e.g., `$fn = 50;`)
///
/// # Examples
///
/// ```ignore
/// use tree_sitter::Node;
/// use openscad_ast::parser::statement::parse_statement;
///
/// // This is typically called by parse_to_ast, not directly
/// let stmt = parse_statement(&node, source)?;
/// ```
pub fn parse_statement(
    node: &Node,
    source: &str,
) -> Result<Option<Statement>, Vec<Diagnostic>> {
    let kind = node.kind();

    // Dispatch based on CST node kind
    if kind == "module_call" {
        // Direct primitive calls: cube, sphere, cylinder
        parse_module_call(node, source)
    } else if kind == "transform_chain" {
        // Transforms with children: translate, rotate, scale
        parse_transform_chain(node, source)
    } else if kind == "var_declaration" {
        // Variable assignments: $fn = 50;
        parse_var_declaration(node, source)
    } else {
        // Ignore other nodes (comments, whitespace, etc.)
        Ok(None)
    }
}
