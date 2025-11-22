//! Tests for the WASM-facing pipeline helpers.

use super::*;

/// Confirms the internal compile helper counts nodes for a trivial cube.
///
/// # Examples
/// ```
/// use wasm::compile_and_count_nodes_internal;
/// assert_eq!(compile_and_count_nodes_internal("cube(1);").unwrap(), 1);
/// ```
#[test]
fn compile_counts_single_cube_node() {
    let count = compile_and_count_nodes_internal("cube(1);").expect("evaluation succeeds");
    assert_eq!(count, 1);
}

/// Ensures invalid source surfaces explicit errors.
#[test]
fn compile_rejects_invalid_source() {
    // Syntax error
    let err = compile_and_count_nodes_internal("cube(1").unwrap_err();
    assert!(!err.to_string().is_empty());
}

/// Tests that compile_and_render produces a valid mesh for cube.
///
/// # Examples
/// ```
/// use wasm::compile_and_render_internal;
/// let mesh = compile_and_render_internal("cube([1, 1, 1]);").unwrap();
/// assert_eq!(mesh.vertex_count(), 8);
/// assert_eq!(mesh.triangle_count(), 12);
/// ```
#[test]
fn compile_and_render_produces_cube_mesh() {
    let mesh = compile_and_render_internal("cube([1, 1, 1]);")
        .expect("compilation succeeds");
    
    assert_eq!(mesh.vertex_count(), 8);
    assert_eq!(mesh.triangle_count(), 12);
    assert_eq!(mesh.vertices().len(), 24); // 8 vertices * 3 components
    assert_eq!(mesh.indices().len(), 36); // 12 triangles * 3 indices
}

/// Tests that compile_and_render returns diagnostics for invalid code.
#[test]
fn compile_and_render_rejects_invalid() {
    let result = compile_and_render_internal("unknown_module();");
    
    assert!(result.is_err());
    let diagnostics = result.unwrap_err();
    assert!(!diagnostics.is_empty());
    assert_eq!(diagnostics[0].severity(), openscad_ast::Severity::Error);
    assert!(diagnostics[0].message().contains("unknown") || diagnostics[0].message().contains("module"));
}

/// Tests that mesh buffers contain valid data.
#[test]
fn compile_and_render_mesh_buffers_valid() {
    let mesh = compile_and_render_internal("cube([2, 2, 2]);")
        .expect("compilation succeeds");
    
    // Verify all vertices are finite
    for &v in &mesh.vertices() {
        assert!(v.is_finite(), "Vertex value should be finite");
    }
    
    // Verify all indices are in range
    let vertex_count = mesh.vertex_count() as u32;
    for &idx in &mesh.indices() {
        assert!(idx < vertex_count, "Index {} out of range", idx);
    }
}
