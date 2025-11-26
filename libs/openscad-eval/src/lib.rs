//! # OpenSCAD Evaluator
//!
//! Evaluates OpenSCAD AST and produces Geometry IR (Intermediate Representation).
//! Handles variables, scopes, special variables ($fn, $fa, $fs), and transforms.
//!
//! ## Architecture
//!
//! ### Native (CLI/Server)
//!
//! ```text
//! OpenSCAD Source → openscad-ast (AST) → openscad-eval (Geometry IR)
//! ```
//!
//! ### Browser (WASM)
//!
//! ```text
//! Serialized CST → openscad-ast (AST) → openscad-eval (Geometry IR)
//! ```
//!
//! ## Usage
//!
//! ### Native
//!
//! ```rust,ignore
//! use openscad_eval::evaluate;
//!
//! let source = "cube(10);";
//! let geometry = evaluate(source)?;
//! ```
//!
//! ### Browser (from serialized CST)
//!
//! ```rust,ignore
//! use openscad_eval::evaluate_from_cst;
//! use openscad_ast::SerializedNode;
//!
//! let cst: SerializedNode = serde_json::from_str(json)?;
//! let geometry = evaluate_from_cst(&cst)?;
//! ```

pub mod context;
pub mod error;
pub mod evaluator;
pub mod ir;
pub mod resolution;

pub use context::EvaluationContext;
pub use error::EvalError;
#[cfg(feature = "native-parser")]
pub use evaluator::evaluate;
pub use evaluator::evaluate_from_cst;
pub use ir::GeometryNode;

// Re-export commonly used types
pub use ir::{BooleanOperation, OffsetAmount};

#[cfg(test)]
mod tests;
