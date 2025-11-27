//! # OpenSCAD Mesh
//!
//! Mesh generation from evaluated geometry.
//!
//! ## Architecture
//!
//! ```text
//! Source → openscad-eval (GeometryNode) → openscad-mesh (Mesh) → wasm
//! ```
//!
//! ## Example
//!
//! ```rust
//! use openscad_mesh::render;
//!
//! let mesh = render("cube(10);").unwrap();
//! assert!(!mesh.vertices.is_empty());
//! ```

pub mod mesh;
pub mod error;
pub mod ops;
pub mod visitor;

// Re-export public API
pub use mesh::Mesh;
pub use error::MeshError;
pub use ops::boolean::{difference, intersection, union};

// =============================================================================
// PUBLIC API
// =============================================================================

/// Render OpenSCAD source code to a mesh.
///
/// This is the main entry point for mesh generation.
///
/// ## Parameters
///
/// - `source`: OpenSCAD source code string
///
/// ## Returns
///
/// `Result<Mesh, MeshError>` - Triangle mesh on success
///
/// ## Example
///
/// ```rust
/// use openscad_mesh::render;
///
/// let mesh = render("cube(10);").unwrap();
/// println!("Vertices: {}", mesh.vertices.len() / 3);
/// println!("Triangles: {}", mesh.indices.len() / 3);
/// ```
pub fn render(source: &str) -> Result<Mesh, MeshError> {
    // Evaluate source to geometry
    let evaluated = openscad_eval::evaluate(source)
        .map_err(|e| MeshError::EvalError(e.to_string()))?;
    
    // Generate mesh from geometry
    visitor::mesh_builder::build_mesh(&evaluated.geometry)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test rendering simple cube.
    #[test]
    fn test_render_cube() {
        let mesh = render("cube(10);").unwrap();
        
        // Cube: 24 vertices (4 per face * 6 faces)
        assert_eq!(mesh.vertices.len(), 72); // 24 * 3 components
        // Cube: 12 triangles (2 per face * 6 faces)
        assert_eq!(mesh.indices.len(), 36); // 12 * 3 indices
        // Same normals count as vertices
        assert_eq!(mesh.normals.len(), 72);
    }

    /// Test rendering centered cube.
    #[test]
    fn test_render_cube_center() {
        let mesh = render("cube(10, center=true);").unwrap();
        
        // Vertices should be centered around origin
        // Check that we have both positive and negative coordinates
        let has_negative = mesh.vertices.iter().any(|&v| v < 0.0);
        let has_positive = mesh.vertices.iter().any(|&v| v > 0.0);
        assert!(has_negative && has_positive);
    }

    /// Test rendering sphere.
    #[test]
    fn test_render_sphere() {
        let mesh = render("sphere(5);").unwrap();
        
        // Should have vertices
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    /// Test rendering translated cube.
    #[test]
    fn test_render_translate() {
        let mesh = render("translate([10, 0, 0]) cube(5);").unwrap();
        
        // All x coordinates should be >= 10
        for i in (0..mesh.vertices.len()).step_by(3) {
            assert!(mesh.vertices[i] >= 10.0, "x={} should be >= 10", mesh.vertices[i]);
        }
    }

    /// Test rendering cylinder.
    #[test]
    fn test_render_cylinder() {
        let mesh = render("cylinder(h=10, r=5);").unwrap();
        
        // Should have vertices and triangles
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
        assert!(!mesh.normals.is_empty());
    }

    /// Test rendering cone (cylinder with r2=0).
    #[test]
    fn test_render_cone() {
        let mesh = render("cylinder(h=10, r1=5, r2=0);").unwrap();
        
        // Should have vertices
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    /// Test rendering rotated cube.
    #[test]
    fn test_render_rotate() {
        let mesh = render("rotate([0, 0, 45]) cube(10);").unwrap();
        
        // Should have vertices
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    /// Test rendering scaled cube.
    #[test]
    fn test_render_scale() {
        let mesh = render("scale([2, 1, 1]) cube(5);").unwrap();
        
        // Should have vertices
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    /// Test rendering difference.
    #[test]
    fn test_render_difference() {
        let mesh = render("difference() { cube(10, center=true); cube(5, center=true); }").unwrap();
        
        // Should have vertices (hollow cube)
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    /// Test rendering intersection.
    #[test]
    fn test_render_intersection() {
        let mesh = render("intersection() { cube(10, center=true); translate([5, 0, 0]) cube(10, center=true); }").unwrap();
        
        // Should have vertices (overlapping region)
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    /// Test variable assignment in rendering.
    #[test]
    fn test_render_with_variable() {
        let mesh = render("size = 10; cube(size);").unwrap();
        
        // Should have vertices
        assert!(!mesh.vertices.is_empty());
        assert_eq!(mesh.vertex_count(), 24); // 4 vertices * 6 faces
    }

    /// Test $fn in sphere rendering.
    #[test]
    fn test_render_sphere_with_fn() {
        // Low resolution sphere
        let mesh_low = render("$fn = 8; sphere(5);").unwrap();
        
        // Higher resolution sphere
        let mesh_high = render("$fn = 16; sphere(5);").unwrap();
        
        // Higher resolution should have more vertices
        assert!(mesh_high.vertex_count() > mesh_low.vertex_count());
    }

    /// Test $fn in cylinder rendering.
    #[test]
    fn test_render_cylinder_with_fn() {
        let mesh = render("cylinder(h=10, r=5, $fn=12);").unwrap();
        
        // Should have vertices
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }
}
