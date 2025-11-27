//! # Mesh Operations
//!
//! Operations on meshes including boolean operations (CSG).

pub mod boolean;

pub use boolean::{difference, intersection, union};
