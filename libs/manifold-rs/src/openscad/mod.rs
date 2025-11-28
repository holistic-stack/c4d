//! # OpenSCAD Compatibility Wrapper
//!
//! Provides 100% OpenSCAD compatibility for $fn/$fa/$fs segment calculation
//! and GeometryNode to Manifold conversion.
//!
//! ## Modules
//!
//! - `segments`: $fn/$fa/$fs → circularSegments conversion
//! - `from_ir`: GeometryNode → Mesh conversion
//!
//! ## OpenSCAD Segment Calculation
//!
//! OpenSCAD uses three special variables to control arc tessellation:
//! - `$fn`: Fixed number of segments (overrides $fa/$fs when > 0)
//! - `$fa`: Minimum angle per segment (default: 12°)
//! - `$fs`: Minimum segment length (default: 2mm)
//!
//! Formula: `max($fn, ceil(360/$fa), ceil(circumference/$fs))`

pub mod segments;
pub mod from_ir;

// Re-export main types
pub use segments::SegmentParams;
