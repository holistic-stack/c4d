//! # OpenSCAD AST Crate
//!
//! Converts Tree-sitter CST (Concrete Syntax Tree) into a typed AST (Abstract Syntax Tree)
//! for OpenSCAD programs. Every node carries source span information for diagnostics.
//!
//! ## Architecture
//!
//! ### Native (CLI/Server)
//!
//! ```text
//! OpenSCAD Source → tree-sitter-openscad-parser (CST) → openscad-ast (AST)
//! ```
//!
//! ### Browser (WASM)
//!
//! ```text
//! OpenSCAD Source → web-tree-sitter (JS) → Serialized CST → openscad-ast (AST)
//! ```
//!
//! ## Usage
//!
//! ### Native Parsing
//!
//! ```rust,ignore
//! use openscad_ast::parse_to_ast;
//!
//! let source = "cube(10);";
//! let statements = parse_to_ast(source)?;
//! ```
//!
//! ### Browser Parsing (from serialized CST)
//!
//! ```rust,ignore
//! use openscad_ast::cst::SerializedNode;
//! use openscad_ast::parse_from_cst;
//!
//! let cst: SerializedNode = serde_json::from_str(json)?;
//! let statements = parse_from_cst(&cst)?;
//! ```
//!
//! ## Design Principles
//!
//! - **Typed AST**: All nodes are strongly typed Rust enums/structs
//! - **Source Mapping**: Every node carries `Span { start, end }` for diagnostics
//! - **No Evaluation**: Pure syntax transformation, no semantic analysis
//! - **Browser-Safe**: CST parser compiles to WASM without native dependencies

pub mod ast;
pub mod cst;
pub mod cst_parser;
pub mod diagnostic;
#[cfg(feature = "native-parser")]
pub mod parser;
pub mod span;

// Re-exports for convenience
pub use ast::*;
pub use cst::SerializedNode;
pub use cst_parser::{parse_from_cst, CstParseError};
pub use diagnostic::{Diagnostic, Severity};
#[cfg(feature = "native-parser")]
pub use parser::parse_to_ast;
pub use span::Span;

#[cfg(test)]
mod tests;
