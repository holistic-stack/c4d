//! # Mesh Operations
//!
//! Browser-safe algorithms for mesh manipulation:
//! - **Boolean Operations**: Union, difference, intersection using BSP trees
//! - **Extrusions**: linear_extrude, rotate_extrude for 2D to 3D conversion
//! - **Hull**: Convex hull using QuickHull algorithm
//! - **Minkowski**: Minkowski sum for rounded shapes
//! - **Offset**: 2D polygon offset using Clipper-style algorithm

pub mod boolean;
pub mod extrude;
pub mod hull;
pub mod minkowski;
pub mod offset;

pub use boolean::{union, difference, intersection};
pub use extrude::{linear_extrude, rotate_extrude, Polygon2D};
pub use hull::hull;
pub use minkowski::minkowski;
pub use offset::{offset_polygon, OffsetParams};
