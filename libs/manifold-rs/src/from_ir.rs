/// Conversion from IR to Manifold mesh.
///
/// This module bridges the evaluator and the geometry kernel.

use crate::{MeshBuffers, Vec3};
use crate::primitives::cube::cube;
use openscad_eval::{Evaluator, InMemoryFilesystem, GeometryNode, EvaluationError};
use openscad_ast::{Diagnostic, Span};

/// Compiles OpenSCAD source code to a mesh.
///
/// This function orchestrates the pipeline:
/// 1. Evaluates source to IR using `openscad-eval`
/// 2. Converts IR nodes to `Manifold` geometry
/// 3. Exports geometry to `MeshBuffers`
///
/// # Arguments
/// * `source` - The OpenSCAD source code
///
/// # Returns
/// * `Ok(MeshBuffers)` - The generated mesh
/// * `Err(Vec<Diagnostic>)` - Diagnostics if compilation fails
pub fn from_source(source: &str) -> Result<MeshBuffers, Vec<Diagnostic>> {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    
    let nodes = evaluator.evaluate_source(source).map_err(|e| {
        match e {
            EvaluationError::AstDiagnostics(diags) => diags,
            _ => {
                let span = Span::new(0, source.len()).unwrap_or_else(|_| Span::new(0, 1).unwrap());
                vec![Diagnostic::error(format!("Evaluation error: {}", e), span)]
            }
        }
    })?;

    if nodes.is_empty() {
        return Ok(MeshBuffers {
            vertices: Vec::new(),
            indices: Vec::new(),
        });
    }

    // For Task 1.1, we process the first node.
    // Future tasks will handle boolean unions of multiple nodes.
    let node = &nodes[0];
    
    let manifold = match node {
        GeometryNode::Cube { size, center, span } => {
             cube(Vec3::new(size.x, size.y, size.z), *center)
                 .map_err(|e| {
                     vec![Diagnostic::error(format!("Manifold error: {}", e), *span)]
                 })?
        }
    };

    Ok(manifold.to_mesh_buffers())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_generation() {
        let mesh = from_source("cube(10);").expect("compilation succeeds");
        assert_eq!(mesh.vertex_count(), 8);
        assert_eq!(mesh.triangle_count(), 12);
    }

    #[test]
    fn test_cube_vector_generation() {
        let mesh = from_source("cube([1, 2, 3]);").expect("compilation succeeds");
        assert_eq!(mesh.vertex_count(), 8);
    }

    #[test]
    fn test_invalid_source() {
        let result = from_source("cube(");
        assert!(result.is_err());
    }
}
