//! Evaluator scaffolding for the Rust OpenSCAD pipeline.
//!
//! This crate wires AST nodes into a typed IR, provides evaluator entry points
//! with stack growth protection, and exposes a filesystem abstraction required
//! by future tasks.
//!
//! ```
//! use openscad_eval::{evaluator::Evaluator, filesystem::InMemoryFilesystem};
//!
//! let fs = InMemoryFilesystem::default();
//! let evaluator = Evaluator::new(fs);
//! let nodes = evaluator.evaluate_source("cube(1);").unwrap();
//! assert_eq!(nodes.len(), 1);
//! ```

pub mod evaluator;
pub mod filesystem;
pub mod ir;

pub use evaluator::{ArgumentError, EvaluationError, Evaluator};
pub use filesystem::{FileSystem, FileSystemError, InMemoryFilesystem};
pub use ir::{GeometryNode, GeometryValidationError};
