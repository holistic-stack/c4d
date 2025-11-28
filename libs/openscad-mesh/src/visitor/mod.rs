//! # Mesh Generation Visitors
//!
//! Visitor pattern for converting geometry to mesh.
//!
//! ## Module Structure (SRP)
//!
//! - `mesh_builder` - Main entry point, dispatches to specialized builders
//! - `primitives_3d` - Cube, sphere, cylinder, polyhedron
//! - `primitives_2d` - Circle, square, polygon (flat at Z=0)
//! - `extrusions` - Linear and rotational extrusion
//! - `transforms` - Translate, rotate, scale, mirror
//! - `booleans` - Union, difference, intersection, hull, minkowski
//! - `ops_2d` - Offset, projection

pub mod booleans;
pub mod extrusions;
pub mod mesh_builder;
pub mod ops_2d;
pub mod primitives_2d;
pub mod primitives_3d;
pub mod transforms;

// Re-export main entry point
pub use mesh_builder::build_mesh;

// Internal helper for child mesh building
pub(crate) use mesh_builder::build_mesh_into;
