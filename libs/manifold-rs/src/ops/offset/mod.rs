//! Offset operation for 2D cross-sections.

use crate::{
    core::{cross_section::CrossSection, vec2::Vec2},
    error::{Error, Result},
};
use cavalier_contours::{
    polyline::{PlineSource, PlineSourceMut},
    polyline::{Polyline, PlineVertex},
};

/// Offsets a 2D cross-section.
///
/// # Arguments
///
/// * `cross_section` - The 2D shape to offset.
/// * `amount` - The offset distance.
/// * `join_type` - Join type (Round, Miter, Chamfer).
/// * `miter_limit` - Limit for miter joins.
pub enum JoinType {
    Round,
    Miter,
    Chamfer,
}

pub fn offset(
    cross_section: &CrossSection,
    amount: f64,
    _join_type: JoinType, // TODO: Use this to configure options
    _miter_limit: f64, // TODO: Use this
) -> Result<CrossSection> {
    // Convert CrossSection to Polyline(s)
    let mut shapes = Vec::new();

    for contour in &cross_section.contours {
        let mut pline = Polyline::new();
        // Force CCW winding by checking area?
        // Or just reverse based on empirical failure.
        // OpenSCAD square is CCW.
        // `cavalier_contours` should behave as documented.
        // Let's force reversal of vertices to check if it fixes expansion.
        for p in contour.iter().rev() {
            pline.add_vertex(PlineVertex::new(p.x, p.y, 0.0));
        }
        pline.set_is_closed(true);
        shapes.push(pline);
    }

    let offset_shapes = shapes.iter()
        .map(|s| s.parallel_offset(amount)) // Use positive amount with reversed input
        .collect::<Vec<_>>();

    // Convert back to CrossSection
    let mut new_contours = Vec::new();

    for shapes_list in offset_shapes {
        for shape in shapes_list {
            let mut contour = Vec::new();
            let len = shape.vertex_count();
            for i in 0..len {
                let v = shape.at(i);
                contour.push(Vec2::new(v.x, v.y));
            }
            new_contours.push(contour);
        }
    }

    Ok(CrossSection::from_contours(new_contours))
}

#[cfg(test)]
mod tests;
