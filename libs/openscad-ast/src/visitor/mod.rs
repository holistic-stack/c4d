//! # AST Visitors
//!
//! Visitor pattern implementations for CST to AST transformation.
//!
//! ## Structure
//!
//! ```text
//! visitor/
//! ├── mod.rs           - This file
//! └── cst_to_ast/      - CST to AST transformation
//!     ├── mod.rs       - Main transformer
//!     ├── statements.rs - Statement transformation
//!     └── expressions.rs - Expression transformation
//! ```

pub mod cst_to_ast;
