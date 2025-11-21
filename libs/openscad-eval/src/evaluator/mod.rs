//! Evaluator entry points bridging AST and IR layers.
//!
//! The initial implementation supports a single cube primitive to satisfy Task
//! 1.1 vertical slice expectations while leaving room for future expansion.

use config::constants::STACKER_STACK_SIZE_BYTES;
use glam::DVec3;
use stacker::maybe_grow;
use thiserror::Error;

use crate::filesystem::FileSystem;
use crate::ir::{GeometryNode, GeometryValidationError};
use openscad_ast::{Diagnostic as AstDiagnostic, Statement};

/// Primary evaluator type parametrized over a filesystem implementation.
///
/// # Examples
/// ```
/// use openscad_eval::{evaluator::Evaluator, filesystem::InMemoryFilesystem};
/// let evaluator = Evaluator::new(InMemoryFilesystem::default());
/// let nodes = evaluator.evaluate_source("cube(1);").unwrap();
/// assert_eq!(nodes.len(), 1);
/// ```
#[derive(Clone, Debug)]
pub struct Evaluator<F: FileSystem + Clone> {
    filesystem: F,
}

impl<F: FileSystem + Clone> Evaluator<F> {
    /// Creates a new evaluator with the provided filesystem.
    pub fn new(filesystem: F) -> Self {
        Self { filesystem }
    }

    /// Evaluates a string of OpenSCAD source code into IR nodes.
    pub fn evaluate_source(&self, source: &str) -> Result<Vec<GeometryNode>, EvaluationError> {
        maybe_grow(
            STACKER_STACK_SIZE_BYTES,
            STACKER_STACK_SIZE_BYTES / 8,
            || self.evaluate_inner(source),
        )
    }

    /// Reads a file through the configured filesystem and evaluates its contents.
    pub fn evaluate_file(&self, path: &str) -> Result<Vec<GeometryNode>, EvaluationError> {
        let content = self
            .filesystem
            .read_to_string(path)
            .map_err(EvaluationError::FileSystem)?;
        self.evaluate_source(&content)
    }

    fn evaluate_inner(&self, source: &str) -> Result<Vec<GeometryNode>, EvaluationError> {
        let trimmed = source.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        // Parse source to AST using the shared AST layer.
        let ast = openscad_ast::parse_to_ast(trimmed)
            .map_err(EvaluationError::AstDiagnostics)?;

        if ast.is_empty() {
            return Err(EvaluationError::UnsupportedSource {
                snippet: trimmed.chars().take(32).collect(),
            });
        }

        ast_to_ir(&ast)
    }
}

/// Converts AST statements into IR geometry nodes.
///
/// The initial implementation supports only `Statement::Cube` nodes and
/// delegates validation to `GeometryNode::cube`.
///
/// # Examples
/// ```
/// use glam::DVec3;
/// use openscad_ast::{CubeSize, Span, Statement};
/// use openscad_eval::ir::GeometryNode;
/// use openscad_eval::evaluator::ast_to_ir;
///
/// let span = Span::new(0, 10).unwrap();
/// let stmt = Statement::Cube {
///     size: CubeSize::Vector([1.0, 2.0, 3.0]),
///     center: Some(true),
///     span,
/// };
/// let nodes = ast_to_ir(&[stmt]).unwrap();
/// assert_eq!(nodes[0].size(), DVec3::new(1.0, 2.0, 3.0));
/// ```
pub fn ast_to_ir(statements: &[Statement]) -> Result<Vec<GeometryNode>, EvaluationError> {
    let mut nodes = Vec::with_capacity(statements.len());

    for statement in statements {
        match statement {
            Statement::Cube { size, center, span } => {
                let vec = size.to_vec3();
                let size_vec = DVec3::new(vec[0], vec[1], vec[2]);
                // Use center from AST, defaulting to false if not specified
                let center_value = center.unwrap_or(false);
                let node = GeometryNode::cube(size_vec, center_value, *span)?;
                nodes.push(node);
            }
        }
    }

    Ok(nodes)
}

/// Errors produced by the evaluator.
#[derive(Debug, Error, PartialEq)]
pub enum EvaluationError {
    /// File could not be read.
    #[error("filesystem error: {0}")]
    FileSystem(#[from] crate::filesystem::FileSystemError),
    /// Invalid argument provided to a primitive.
    #[error("invalid argument: {0}")]
    InvalidArgument(#[from] ArgumentError),
    /// Geometry node failed validation.
    #[error("geometry error: {0}")]
    Geometry(#[from] GeometryValidationError),
    /// AST layer reported diagnostics for the source program.
    #[error("AST diagnostics: {0:?}")]
    AstDiagnostics(Vec<AstDiagnostic>),
    /// Source contains constructs not yet supported in Task 1.1 scaffolding.
    #[error("unsupported source snippet: {snippet}")]
    UnsupportedSource { snippet: String },
}

/// Argument parsing errors for primitives.
#[derive(Debug, Error, PartialEq)]
pub enum ArgumentError {
    /// Provided component count was not exactly three for vector arguments.
    #[error("expected 3 components, got {0}")]
    InvalidComponentCount(usize),
    /// Text could not be parsed into numeric values.
    #[error("invalid numeric literal")]
    InvalidNumber,
}

#[cfg(test)]
mod tests;
