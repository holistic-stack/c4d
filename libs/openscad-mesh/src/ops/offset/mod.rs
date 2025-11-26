//! # 2D Polygon Offset
//!
//! Computes offset (inset/outset) of 2D polygons.
//! Browser-safe implementation in pure Rust.
//!
//! ## Algorithm Overview
//!
//! Uses a simplified Clipper-style algorithm:
//! 1. For each edge, compute parallel offset edge
//! 2. Find intersections of adjacent offset edges
//! 3. Handle miter/round/square joins
//!
//! ## OpenSCAD Compatibility
//!
//! Matches `offset(r=radius)` or `offset(delta=distance)`:
//! - Positive values expand the polygon
//! - Negative values shrink the polygon
//! - `chamfer` parameter controls corner style

#[cfg(test)]
mod tests;

use crate::ops::extrude::Polygon2D;
use config::constants::EPSILON;
use glam::DVec2;

/// Parameters for offset operation.
#[derive(Debug, Clone)]
pub struct OffsetParams {
    /// Offset amount (positive = expand, negative = shrink)
    pub amount: f64,
    /// Use chamfered corners instead of sharp
    pub chamfer: bool,
}

impl Default for OffsetParams {
    fn default() -> Self {
        Self {
            amount: 1.0,
            chamfer: false,
        }
    }
}

/// Computes the offset of a 2D polygon.
///
/// # Arguments
///
/// * `polygon` - The polygon to offset
/// * `params` - Offset parameters
///
/// # Returns
///
/// A new polygon with the offset applied.
///
/// # Example
///
/// ```rust,ignore
/// use openscad_mesh::ops::offset::{offset_polygon, OffsetParams};
///
/// let square = Polygon2D::square(DVec2::splat(10.0), true);
/// let params = OffsetParams { amount: 1.0, chamfer: false };
/// let expanded = offset_polygon(&square, &params)?;
/// ```
pub fn offset_polygon(polygon: &Polygon2D, params: &OffsetParams) -> Result<Polygon2D, String> {
    if polygon.vertex_count() < 3 {
        return Err("Polygon must have at least 3 vertices".to_string());
    }

    if params.amount.abs() < EPSILON {
        return Ok(polygon.clone());
    }

    let vertices = &polygon.outer;
    let n = vertices.len();
    let mut offset_vertices = Vec::with_capacity(n);

    for i in 0..n {
        let prev = vertices[(i + n - 1) % n];
        let curr = vertices[i];
        let next = vertices[(i + 1) % n];

        // Compute edge directions
        let edge1 = curr - prev;
        let edge2 = next - curr;

        // Compute outward normals (perpendicular to edges, pointing outward for CCW polygon)
        // For CCW polygon, outward normal is (-dy, dx)
        let normal1 = DVec2::new(edge1.y, -edge1.x).normalize();
        let normal2 = DVec2::new(edge2.y, -edge2.x).normalize();

        // Average normal for the corner
        let avg_normal = normal1 + normal2;
        let avg_len = avg_normal.length();
        
        if avg_len < EPSILON {
            // Normals are opposite - use either one
            offset_vertices.push(curr + normal1 * params.amount);
            continue;
        }
        
        let avg_normal = avg_normal / avg_len;

        // Compute offset point
        // For sharp corners, we need to account for the angle
        let dot = normal1.dot(normal2);
        let scale = if dot.abs() < 0.999 {
            // Not parallel - compute proper offset using miter formula
            1.0 / (1.0 + dot).max(0.1).sqrt()
        } else {
            1.0
        };

        let offset_point = curr + avg_normal * params.amount * scale;

        if params.chamfer && dot < 0.5 {
            // Add chamfer points for sharp corners
            let offset1 = curr + normal1 * params.amount;
            let offset2 = curr + normal2 * params.amount;
            offset_vertices.push(offset1);
            offset_vertices.push(offset2);
        } else {
            offset_vertices.push(offset_point);
        }
    }

    // Remove self-intersections for negative offsets
    if params.amount < 0.0 {
        offset_vertices = remove_self_intersections(&offset_vertices);
    }

    if offset_vertices.len() < 3 {
        return Err("Offset resulted in degenerate polygon".to_string());
    }

    Ok(Polygon2D::new(offset_vertices))
}

/// Removes self-intersections from a polygon (simplified).
fn remove_self_intersections(vertices: &[DVec2]) -> Vec<DVec2> {
    // Simplified: just remove vertices that would cause backtracking
    let mut result: Vec<DVec2> = Vec::with_capacity(vertices.len());
    
    for &v in vertices {
        // Skip if this vertex would cause a self-intersection
        if result.len() >= 2 {
            let prev: DVec2 = result[result.len() - 1];
            let prev2: DVec2 = result[result.len() - 2];
            
            let edge1: DVec2 = prev - prev2;
            let edge2: DVec2 = v - prev;
            
            // Check if we're backtracking (cross product sign change)
            let cross = edge1.x * edge2.y - edge1.y * edge2.x;
            if cross < -EPSILON {
                // Backtracking - skip this vertex
                continue;
            }
        }
        result.push(v);
    }

    result
}
