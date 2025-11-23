//! Evaluator entry points bridging AST and IR layers.
//!
//! The initial implementation supports a single cube primitive to satisfy Task
//! 1.1 vertical slice expectations while leaving room for future expansion.

use config::constants::STACKER_STACK_SIZE_BYTES;
use glam::{DMat4, DVec3, DVec2};
use stacker::maybe_grow;
use thiserror::Error;

use crate::filesystem::FileSystem;
use crate::ir::{GeometryNode, GeometryValidationError};
use openscad_ast::{self, Diagnostic as AstDiagnostic, Span, Statement};
use self::context::EvaluationContext;
pub mod resolution;
use resolution::compute_segments;

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
                    let effective_fn = fn_.unwrap_or(context.get_fn());
                    let effective_fa = fa.unwrap_or(context.get_fa());
                    let effective_fs = fs.unwrap_or(context.get_fs());

                    let segments = compute_segments(*radius, effective_fn, effective_fa, effective_fs);

                    let node = GeometryNode::sphere(*radius, segments, *span)?;
                    nodes.push(node);
                }
                Statement::Cylinder { height, r1, r2, center, fa, fs, fn_, span } => {
                     let effective_fn = fn_.unwrap_or(context.get_fn());
                    let effective_fa = fa.unwrap_or(context.get_fa());
                    let effective_fs = fs.unwrap_or(context.get_fs());

                    // Use max radius for resolution calculation
                    let max_radius = r1.max(*r2);
                    let segments = compute_segments(max_radius, effective_fn, effective_fa, effective_fs);

                    let node = GeometryNode::cylinder(*height, *r1, *r2, *center, segments, *span)?;
                    nodes.push(node);
                }
                Statement::Square { size, center, span } => {
                    let vec = size.to_vec2();
                    let size_vec = DVec2::new(vec[0], vec[1]);
                    let node = GeometryNode::square(size_vec, *center, *span)?;
                    nodes.push(node);
                }
                Statement::Circle { radius, fa, fs, fn_, span } => {
                    let effective_fn = fn_.unwrap_or(context.get_fn());
                    let effective_fa = fa.unwrap_or(context.get_fa());
                    let effective_fs = fs.unwrap_or(context.get_fs());

                    let segments = compute_segments(*radius, effective_fn, effective_fa, effective_fs);

                    let node = GeometryNode::circle(*radius, segments, *span)?;
                    nodes.push(node);
                }
                Statement::Polygon { points, paths, convexity, span } => {
                     let points_vec: Vec<DVec2> = points.iter().map(|p| DVec2::new(p[0], p[1])).collect();
                     let paths_vec = paths.clone().unwrap_or_else(|| {
                         // Default path is 0..N-1
                         vec![(0..points.len()).collect()]
                     });

                     let node = GeometryNode::polygon(points_vec, paths_vec, *convexity, *span)?;
                     nodes.push(node);
                }
                Statement::Polyhedron { points, faces, convexity, span } => {
                    let points_vec: Vec<DVec3> = points.iter().map(|p| DVec3::new(p[0], p[1], p[2])).collect();
                    let node = GeometryNode::polyhedron(points_vec, faces.clone(), *convexity, *span)?;
                    nodes.push(node);
                }
                Statement::Translate { vector, child, span } => {
                    let translation = DVec3::from_array(*vector);
                    let matrix = DMat4::from_translation(translation);
                    nodes.extend(self.wrap_child_with_transform(child, *span, context, matrix)?);
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

                    nodes.extend(self.wrap_child_with_transform(child, *span, context, rotation)?);
                }
                Statement::Scale { vector, child, span } => {
                    let scale = DVec3::from_array(*vector);
                    let matrix = DMat4::from_scale(scale);
                    nodes.extend(self.wrap_child_with_transform(child, *span, context, matrix)?);
                }
            }
        }

        Ok(nodes)
    }
}

impl<F: FileSystem + Clone> Evaluator<F> {
    /// Evaluates the child statements of a transform node and wraps each in a `GeometryNode::Transform`.
    ///
    /// # Examples
    /// ```
    /// use openscad_eval::{evaluator::Evaluator, filesystem::InMemoryFilesystem};
    /// let evaluator = Evaluator::new(InMemoryFilesystem::default());
    /// let nodes = evaluator
    ///     .evaluate_source("translate([1,0,0]) cube(1);")
    ///     .unwrap();
    /// assert!(matches!(nodes[0], openscad_eval::ir::GeometryNode::Transform { .. }));
    /// ```
    fn wrap_child_with_transform(
        &self,
        child: &Box<Statement>,
        span: Span,
        context: &mut EvaluationContext,
        matrix: DMat4,
    ) -> Result<Vec<GeometryNode>, EvaluationError> {
        let child_stmt = child.as_ref().clone();
        let child_nodes = self.evaluate_statements(&[child_stmt], context)?;
        Ok(child_nodes
            .into_iter()
            .map(|child_node| GeometryNode::Transform {
                matrix,
                child: Box::new(child_node),
                span,
            })
            .collect())
    }
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
#[cfg(test)]
mod arguments_tests;
