//! Minimal OpenSCAD AST definitions shared between parser and evaluator.
//!
//! `openscad-ast` focuses on strongly typed nodes with spans so downstream
//! crates can reason about diagnostics deterministically.
//!
//! ```
//! use openscad_ast::parse_to_ast;
//!
//! let ast = parse_to_ast("cube(10);").unwrap();
//! assert_eq!(ast.len(), 1);
//! ```

pub mod ast_types;
pub mod diagnostic;
pub mod nodes;
pub mod parser;

pub use ast_types::{CubeSize, Statement};
pub use diagnostic::{Diagnostic, Severity};
pub use nodes::{AstMetadata, AstNode, Span, SpanError};
pub use parser::parse_to_ast;
