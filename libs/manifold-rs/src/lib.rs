//! # Manifold-RS
//!
//! Pure Rust port of [Manifold-3D](https://github.com/elalish/manifold) for
//! browser-safe CSG (Constructive Solid Geometry) operations.
//!
//! ## Overview
//!
//! This crate provides robust 3D mesh operations with exact boolean algorithms,
//! designed for WebAssembly compilation without C dependencies.
//!
//! ## Architecture
//!
//! ```text
//! OpenSCAD Source
//!       ↓
//! openscad-eval (GeometryNode)
//!       ↓
//! manifold-rs
//!   ├─ OpenSCAD Wrapper ($fn/$fa/$fs → circularSegments)
//!   ├─ Manifold (3D solid operations)
//!   ├─ CrossSection (2D polygon operations)
//!   └─ Mesh (output format)
//!       ↓
//! wasm (Float32Array/Uint32Array)
//! ```
//!
//! ## Example
//!
//! ```rust
//! use manifold_rs::render;
//!
//! // Render OpenSCAD code to mesh
//! let mesh = render("cube(10);").unwrap();
//! assert!(!mesh.vertices.is_empty());
//! ```
//!
//! ## Browser Safety
//!
//! This crate is designed for WebAssembly:
//! - No C dependencies
//! - No file system access
//! - No WASI requirements
//! - Pure Rust algorithms

// =============================================================================
// MODULE DECLARATIONS
// =============================================================================

/// Error types for manifold operations.
pub mod error;

/// Output mesh format with vertices, indices, and normals.
pub mod mesh;

/// 3D solid operations (Manifold-3D port).
pub mod manifold;

/// 2D polygon operations for extrusions.
pub mod cross_section;

/// OpenSCAD compatibility wrapper for $fn/$fa/$fs.
pub mod openscad;

// =============================================================================
// RE-EXPORTS
// =============================================================================

pub use error::ManifoldError;
pub use mesh::Mesh;
pub use manifold::Manifold;
pub use cross_section::CrossSection;
pub use openscad::SegmentParams;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Render OpenSCAD source code to a mesh.
///
/// This is the main entry point for mesh generation. It evaluates the source
/// code and converts the geometry tree to a triangle mesh.
///
/// ## Parameters
///
/// - `source`: OpenSCAD source code string
///
/// ## Returns
///
/// `Result<Mesh, ManifoldError>` - Triangle mesh on success
///
/// ## Example
///
/// ```rust
/// use manifold_rs::render;
///
/// let mesh = render("cube(10);").unwrap();
/// println!("Vertices: {}", mesh.vertex_count());
/// println!("Triangles: {}", mesh.triangle_count());
/// ```
///
/// ## Errors
///
/// Returns `ManifoldError::EvalError` if source code evaluation fails.
/// Returns `ManifoldError::GeometryError` if mesh generation fails.
pub fn render(source: &str) -> Result<Mesh, ManifoldError> {
    // Step 1: Evaluate source to geometry using openscad-eval
    let evaluated = openscad_eval::evaluate(source)
        .map_err(|e| ManifoldError::EvalError(e.to_string()))?;
    
    // Step 2: Convert GeometryNode to Mesh using OpenSCAD wrapper
    openscad::from_ir::geometry_to_mesh(&evaluated.geometry)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test rendering simple cube.
    ///
    /// Verifies that a basic cube generates the expected vertex and triangle
    /// counts (24 vertices = 4 per face × 6 faces, 12 triangles = 2 per face).
    #[test]
    fn test_render_cube() {
        let mesh = render("cube(10);").unwrap();
        
        // Cube: 24 vertices (4 per face × 6 faces)
        assert_eq!(mesh.vertex_count(), 24);
        // Cube: 12 triangles (2 per face × 6 faces)
        assert_eq!(mesh.triangle_count(), 12);
    }

    /// Test rendering centered cube.
    ///
    /// Verifies that center=true produces geometry centered at origin.
    #[test]
    fn test_render_cube_centered() {
        let mesh = render("cube(10, center=true);").unwrap();
        
        // Should have both positive and negative coordinates
        let has_negative = mesh.vertices.iter().any(|&v| v < 0.0);
        let has_positive = mesh.vertices.iter().any(|&v| v > 0.0);
        assert!(has_negative && has_positive);
    }

    /// Test rendering sphere with $fn parameter.
    ///
    /// Verifies that higher $fn produces more vertices.
    #[test]
    fn test_render_sphere_with_fn() {
        let mesh_low = render("$fn = 8; sphere(5);").unwrap();
        let mesh_high = render("$fn = 16; sphere(5);").unwrap();
        
        assert!(mesh_high.vertex_count() > mesh_low.vertex_count());
    }

    /// Test validation test case with boolean operations.
    ///
    /// The main acceptance test for the pipeline.
    #[test]
    fn test_validation_test_case() {
        let source = r#"
            translate([-24,0,0]) {
                union() {
                    cube(15, center=true);
                    sphere(10);
                }
            }
            intersection() {
                cube(15, center=true);
                sphere(10);
            }
            translate([24,0,0]) {
                difference() {
                    cube(15, center=true);
                    sphere(10);
                }
            }
        "#;
        
        let mesh = render(source).unwrap();
        
        // Should produce substantial geometry
        assert!(mesh.vertex_count() > 100);
        assert!(mesh.triangle_count() > 50);
    }
}
