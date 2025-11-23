use crate::Manifold;
use crate::error::Result;
use crate::BooleanOp;
use crate::primitives::triangulate::manifold_from_contours;

mod cross_section;
use cross_section::boolean_2d;

#[cfg(test)]
mod tests;

pub fn boolean(a: &Manifold, b: &Manifold, op: BooleanOp) -> Result<Manifold> {
    // 1. Check if both operands are 2D cross-sections (flat on Z=0)
    let cs_a = a.to_cross_section();
    let cs_b = b.to_cross_section();

    if let (Some(cs_a), Some(cs_b)) = (cs_a, cs_b) {
        // Perform 2D boolean
        match boolean_2d(&cs_a, &cs_b, op) {
            Ok(res_cs) => {
                return manifold_from_contours(
                    res_cs.contours.iter().map(|contour| {
                        contour.iter().map(|p| crate::Vec3::new(p.x, p.y, 0.0)).collect()
                    }).collect()
                ).map_err(|e| crate::error::Error::MeshGeneration(e));
            }
            Err(e) => {
                 // Fallback to 3D or just error?
                 // If 2D boolean fails, maybe we should try 3D?
                 // But for now, report error.
                 return Err(crate::error::Error::BooleanError(e));
            }
        }
    }

    // 2. If not 2D, use 3D boolean (only Union implemented for now)
    match op {
        BooleanOp::Union => {
            // Naive union: Append meshes
            // Note: This does not resolve intersections!
            let mut result = a.clone();

            let vertex_offset = result.vertices.len() as u32;
            let half_edge_offset = result.half_edges.len() as u32;
            let face_offset = result.faces.len() as u32;

            // Append vertices
            result.vertices.extend(b.vertices.iter().map(|v| {
                let mut v_new = *v;
                if v_new.first_edge != u32::MAX {
                    v_new.first_edge += half_edge_offset;
                }
                v_new
            }));

            // Append half-edges
            result.half_edges.extend(b.half_edges.iter().map(|he| {
                let mut he_new = *he;
                he_new.start_vert += vertex_offset;
                he_new.end_vert += vertex_offset;
                he_new.next_edge += half_edge_offset;
                if he_new.pair_edge != u32::MAX {
                    he_new.pair_edge += half_edge_offset;
                }
                he_new.face += face_offset;
                he_new
            }));

            // Append faces
            result.faces.extend(b.faces.iter().map(|f| {
                let mut f_new = *f;
                if f_new.first_edge != u32::MAX {
                    f_new.first_edge += half_edge_offset;
                }
                f_new
            }));

            Ok(result)
        }
        BooleanOp::Difference | BooleanOp::Intersection => {
            Err(crate::error::Error::BooleanError(
                "3D Difference and Intersection not yet implemented (requires CSG library)".to_string()
            ))
        }
    }
}
