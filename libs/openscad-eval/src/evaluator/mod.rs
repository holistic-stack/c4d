//! Evaluator entry points bridging AST and IR layers.
//!
//! The initial implementation supports a single cube primitive to satisfy Task
//! 1.1 vertical slice expectations while leaving room for future expansion.

use config::constants::STACKER_STACK_SIZE_BYTES;
use glam::{DMat4, DVec3};
use stacker::maybe_grow;
use thiserror::Error;

use crate::filesystem::FileSystem;
use crate::ir::{GeometryNode, GeometryValidationError};
use openscad_ast::{Diagnostic as AstDiagnostic, Statement};
use self::context::EvaluationContext;

pub mod context;

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

        let mut context = EvaluationContext::default();
        self.evaluate_statements(&ast, &mut context)
    }

    fn evaluate_statements(
        &self,
        statements: &[Statement],
        context: &mut EvaluationContext,
    ) -> Result<Vec<GeometryNode>, EvaluationError> {
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
                Statement::Assignment { name, value, .. } => {
                    context.set_variable(name, *value);
                }
                Statement::Sphere { radius, fa, fs, fn_, span } => {
                    // Update context with local variables if present
                    // (OpenSCAD scoping rules are complex, but for primitives, params override globals)
                    // For now, we assume $fn/$fa/$fs passed as args should influence the resolution calculation.
                    // However, the current EvaluationContext manages global state.
                    // For primitives, we should probably compute the effective resolution locally
                    // or temporarily override the context.
                    // Since the context is mutable, we can override and restore, but `evaluate_statements` is sequential.
                    // Primitives are terminals in this simple evaluator.

                    // Better approach: use context to resolve segments count.
                    // OpenSCAD logic for fragments:
                    // if $fn > 0: segments = $fn (if >= 3)
                    // else: segments = ceil(max(min(360/$fa, r*2*PI/$fs), 5))

                    let effective_fn = fn_.unwrap_or(context.get_fn());
                    let effective_fa = fa.unwrap_or(context.get_fa());
                    let effective_fs = fs.unwrap_or(context.get_fs());

                    let segments = calculate_segments(*radius, effective_fn, effective_fa, effective_fs);

                    let node = GeometryNode::sphere(*radius, segments, *span)?;
                    nodes.push(node);
                }
                Statement::Translate { vector, child, span } => {
                    let translation = DVec3::from_array(*vector);
                    let matrix = DMat4::from_translation(translation);

                    let child_stmt = child.as_ref().clone();
                    let child_nodes = self.evaluate_statements(&[child_stmt], context)?;
                    for child_node in child_nodes {
                        nodes.push(GeometryNode::Transform {
                            matrix,
                            child: Box::new(child_node),
                            span: *span,
                        });
                    }
                }
                Statement::Rotate { vector, child, span } => {
                    // OpenSCAD rotate is Euler angles in degrees.
                    // Order: X then Y then Z (if vector).
                    let degs = DVec3::from_array(*vector);
                    let rads = DVec3::new(degs.x.to_radians(), degs.y.to_radians(), degs.z.to_radians());

                    // Matrix multiplication order for column vectors (glam): M = Mz * My * Mx
                    let rotation = DMat4::from_rotation_z(rads.z)
                        * DMat4::from_rotation_y(rads.y)
                        * DMat4::from_rotation_x(rads.x);

                    let matrix = rotation;

                    let child_stmt = child.as_ref().clone();
                    let child_nodes = self.evaluate_statements(&[child_stmt], context)?;
                    for child_node in child_nodes {
                        nodes.push(GeometryNode::Transform {
                            matrix,
                            child: Box::new(child_node),
                            span: *span,
                        });
                    }
                }
                Statement::Scale { vector, child, span } => {
                    let scale = DVec3::from_array(*vector);
                    let matrix = DMat4::from_scale(scale);

                    let child_stmt = child.as_ref().clone();
                    let child_nodes = self.evaluate_statements(&[child_stmt], context)?;
                    for child_node in child_nodes {
                        nodes.push(GeometryNode::Transform {
                            matrix,
                            child: Box::new(child_node),
                            span: *span,
                        });
                    }
                }
            }
        }

        Ok(nodes)
    }
}

fn calculate_segments(radius: f64, fn_val: u32, fa_val: f64, fs_val: f64) -> u32 {
    if fn_val > 0 {
        if fn_val >= 3 { fn_val } else { 3 }
    } else {
        let segments_fa = if fa_val > 0.0 { 360.0 / fa_val } else { 0.0 };
        let segments_fs = if fs_val > 0.0 { (radius * 2.0 * std::f64::consts::PI) / fs_val } else { 0.0 };

        let segments = segments_fa.min(segments_fs);
        let segments = segments.ceil() as u32;
        if segments < 5 { 5 } else { segments }
    }
}

// ast_to_ir was removed and integrated into Evaluator::evaluate_statements

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
#[cfg(test)]
mod arguments_tests;
