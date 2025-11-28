//! # Mesh Operations
//!
//! Operations on meshes including boolean operations (CSG), convex hull,
//! and Minkowski sum.
//!
//! ## Available Operations
//!
//! - `boolean` - Union, difference, intersection (BSP-based CSG)
//! - `hull` - Convex hull using QuickHull algorithm
//! - `minkowski` - Minkowski sum approximation

pub mod boolean;
pub mod hull;
pub mod minkowski;

pub use boolean::{difference, intersection, union};
pub use hull::convex_hull;
pub use minkowski::minkowski_sum;
