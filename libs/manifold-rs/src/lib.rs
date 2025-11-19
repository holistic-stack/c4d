//! Manifold geometry kernel for Rust OpenSCAD pipeline
//!
//! This crate provides the core geometry kernel with index-based half-edge
//! mesh representation, following the C++ Manifold library architecture.

pub mod config;
pub mod core;
pub mod primitives;

pub use config::*;
pub use core::*;
pub use primitives::*;

use openscad_eval::{evaluate, EvaluatedGeometry};

/// Processes OpenSCAD source code and returns a triangulated mesh as a flat vector of coordinates.
pub fn process_openscad(source: &str) -> Result<Vec<f64>, String> {
    let evaluated_geometry = evaluate(source)?;

    match evaluated_geometry {
        EvaluatedGeometry::Cube(cube_params) => {
            let center = if cube_params.center {
                Vec3::new(0.0, 0.0, 0.0)
            } else {
                Vec3::new(
                    cube_params.size[0] / 2.0,
                    cube_params.size[1] / 2.0,
                    cube_params.size[2] / 2.0,
                )
            };
            let size = Vec3::new(
                cube_params.size[0],
                cube_params.size[1],
                cube_params.size[2],
            );

            let cube = Cube::from_center_size(center, size);
            
            match cube.to_mesh() {
                Ok(mesh) => Ok(mesh.triangulate()),
                Err(e) => Err(format!("Error creating cube mesh: {:?}", e)),
            }
        }
    }
}
