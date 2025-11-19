//! Core geometry data structures and algorithms
//!
//! This module contains the fundamental data structures for representing
//! 3D geometry, including vector math and half-edge mesh representation.

pub mod vec3;
pub mod half_edge;

pub use vec3::*;
pub use half_edge::*;