//! Modular parser entry point orchestrating CST → AST conversion.
//!
//! This module wires the new SRP-compliant parser layout described in
//! `specs/split-parser/plan.md`. For now, all logic still lives inside
//! `legacy.rs` while the refactor proceeds incrementally via TDD.
//!
//! # Examples
//! ```
//! use openscad_ast::parse_to_ast;
//!
//! let ast = parse_to_ast("cube(10);").expect("parse succeeds");
//! assert_eq!(ast.len(), 1);
//! ```
+
+mod assignments;
+pub(crate) mod arguments;
+mod legacy;
+mod module_call;
+mod statement;
+mod transform_chain;
+
+#[cfg(test)]
+pub mod tests;
+
+pub use legacy::parse_to_ast;
