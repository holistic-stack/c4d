//! # OpenSCAD Mesh
//!
//! Browser-safe mesh generation for OpenSCAD geometry.
//! Converts Geometry IR from `openscad-eval` into triangle meshes.
//!
//! ## Architecture
//!
//! ```text
//! openscad-eval (Geometry IR) → openscad-mesh (Mesh)
//! ```
//!
//! ## Algorithms
//!
//! All algorithms are browser-safe (pure Rust, no native dependencies):
//! - **Boolean Operations**: BSP trees (csg.js algorithm)
//! - **Triangulation**: Ear clipping
//! - **Hull**: QuickHull
//! - **Primitives**: Custom mesh generation
//!
//! ## Usage
//!
//! ```rust,ignore
//! use openscad_mesh::compile_and_render;
//!
//! let source = "cube(10);";
//! let mesh = compile_and_render(source)?;
//! ```

pub mod error;
pub mod from_ir;
pub mod mesh;
pub mod ops;
pub mod primitives;

pub use error::MeshError;
pub use mesh::Mesh;
pub use ops::boolean::{union, difference, intersection};

/// Compiles OpenSCAD source and renders to a mesh.
///
/// This is the main entry point for the mesh generation pipeline.
/// Only available with the `native-parser` feature.
///
/// # Arguments
///
/// * `source` - OpenSCAD source code
///
/// # Returns
///
/// A mesh containing vertices and triangle indices.
///
/// # Example
///
/// ```rust,ignore
/// use openscad_mesh::compile_and_render;
///
/// let mesh = compile_and_render("cube(10);")?;
/// assert_eq!(mesh.vertex_count(), 8);
/// assert_eq!(mesh.triangle_count(), 12);
/// ```
#[cfg(feature = "native-parser")]
pub fn compile_and_render(source: &str) -> Result<Mesh, MeshError> {
    let geometry = openscad_eval::evaluate(source)?;
    from_ir::geometry_to_mesh(&geometry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_compile_cube() {
        let mesh = compile_and_render("cube(10);").unwrap();
        assert_eq!(mesh.vertex_count(), 8);
        assert_eq!(mesh.triangle_count(), 12);
    }

    #[test]
    fn test_compile_sphere() {
        let mesh = compile_and_render("sphere(5);").unwrap();
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_compile_translate() {
        let mesh = compile_and_render("translate([10,0,0]) cube(5);").unwrap();
        assert_eq!(mesh.vertex_count(), 8);
        // Check bounding box is shifted
        let (min, max) = mesh.bounding_box();
        assert!(min.x >= 10.0);
        assert!(max.x <= 15.0);
    }

    /// Performance test for the target validation test case.
    /// This test measures compilation time for the boolean operations demo.
    #[test]
    fn test_performance_target_validation() {
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

        // Warm-up run
        let _ = compile_and_render(source).unwrap();

        // Timed run
        let start = Instant::now();
        let mesh = compile_and_render(source).unwrap();
        let elapsed = start.elapsed();

        println!("Target validation compile time: {:?}", elapsed);
        println!("Vertices: {}, Triangles: {}", mesh.vertex_count(), mesh.triangle_count());

        // Ensure we get valid output
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);

        // Performance target: should complete in under 500ms
        assert!(elapsed.as_millis() < 500, "Compilation took too long: {:?}", elapsed);
    }

    /// Performance test for complex nested operations.
    #[test]
    fn test_performance_complex_scene() {
        let source = r#"
            // Grid of cubes with boolean operations
            for (x = [-20, 0, 20]) {
                for (y = [-20, 0, 20]) {
                    translate([x, y, 0]) {
                        difference() {
                            cube(8, center=true);
                            sphere(5);
                        }
                    }
                }
            }
        "#;

        let start = Instant::now();
        let mesh = compile_and_render(source).unwrap();
        let elapsed = start.elapsed();

        println!("Complex scene compile time: {:?}", elapsed);
        println!("Vertices: {}, Triangles: {}", mesh.vertex_count(), mesh.triangle_count());

        assert!(mesh.vertex_count() > 0);
    }
}
