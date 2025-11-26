//! # Primitives
//!
//! Mesh generation for OpenSCAD primitives (cube, sphere, cylinder, etc.).

pub mod cube;
pub mod sphere;
pub mod cylinder;

pub use cube::create_cube;
pub use sphere::create_sphere;
pub use cylinder::create_cylinder;
