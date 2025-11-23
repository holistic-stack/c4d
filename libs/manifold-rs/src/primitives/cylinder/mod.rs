//! Cylinder primitive.

use crate::{
    core::{ds::Vertex, vec3::Vec3},
    error::{Error, Result},
    Manifold,
    ops::utils::{add_triangle, stitch_mesh},
};

/// Creates a cylinder (or cone).
///
/// # Arguments
/// * `height` - The height of the cylinder along Z.
/// * `radius_bottom` - Radius at Z=0 (or bottom if centered).
/// * `radius_top` - Radius at Z=height (or top if centered).
/// * `center` - Whether to center along Z.
/// * `segments` - Number of radial segments.
///
/// # Returns
/// * `Ok(Manifold)` - The cylinder mesh.
pub fn cylinder(
    height: f64,
    radius_bottom: f64,
    radius_top: f64,
    center: bool,
    segments: u32,
) -> Result<Manifold> {
    if height <= 0.0 {
        return Err(Error::InvalidGeometry {
            message: "Cylinder height must be positive".to_string(),
        });
    }
    if radius_bottom < 0.0 || radius_top < 0.0 {
        return Err(Error::InvalidGeometry {
            message: "Cylinder radii must be non-negative".to_string(),
        });
    }
    if radius_bottom == 0.0 && radius_top == 0.0 {
        return Err(Error::InvalidGeometry {
            message: "Cylinder cannot have both radii zero".to_string(),
        });
    }

    let segments = segments.max(3);
    let z_offset = if center { -height / 2.0 } else { 0.0 };
    let step_angle = std::f64::consts::TAU / segments as f64;

    let mut vertices = Vec::new();
    let mut faces = Vec::new();
    let mut half_edges = Vec::new();

    // Helper to create ring vertices
    let create_ring = |r: f64, z: f64, verts: &mut Vec<Vertex>| {
        let start_idx = verts.len();
        for i in 0..segments {
            let theta = i as f64 * step_angle;
            let (sin, cos) = theta.sin_cos();
            verts.push(Vertex {
                position: Vec3::new(r * cos, r * sin, z),
                first_edge: 0,
            });
        }
        start_idx
    };

    if radius_bottom > 0.0 && radius_top > 0.0 {
        // Cylinder/Frustum
        let bottom_start = create_ring(radius_bottom, z_offset, &mut vertices);
        let top_start = create_ring(radius_top, z_offset + height, &mut vertices);

        // Side Faces (Quads -> 2 Tris)
        for i in 0..segments {
            let j = (i + 1) % segments;

            let v_curr_bot = (bottom_start as u32) + i;
            let v_next_bot = (bottom_start as u32) + j;
            let v_curr_top = (top_start as u32) + i;
            let v_next_top = (top_start as u32) + j;

            add_triangle(&mut faces, &mut half_edges, v_curr_bot, v_next_bot, v_curr_top);
            add_triangle(&mut faces, &mut half_edges, v_next_bot, v_next_top, v_curr_top);
        }

        // Bottom Cap
        for i in 1..segments - 1 {
            let v0 = bottom_start as u32;
            let v1 = bottom_start as u32 + i + 1;
            let v2 = bottom_start as u32 + i;
            add_triangle(&mut faces, &mut half_edges, v0, v1, v2);
        }

        // Top Cap
        for i in 1..segments - 1 {
            let v0 = top_start as u32;
            let v1 = top_start as u32 + i;
            let v2 = top_start as u32 + i + 1;
            add_triangle(&mut faces, &mut half_edges, v0, v1, v2);
        }

    } else if radius_top == 0.0 {
        // Cone (Tip at top)
        let bottom_start = create_ring(radius_bottom, z_offset, &mut vertices);
        // Apex
        let apex_idx = vertices.len() as u32;
        vertices.push(Vertex {
            position: Vec3::new(0.0, 0.0, z_offset + height),
            first_edge: 0,
        });

        // Sides (Triangles)
        for i in 0..segments {
            let j = (i + 1) % segments;
            let v_curr = (bottom_start as u32) + i;
            let v_next = (bottom_start as u32) + j;

            // Triangle: curr -> next -> apex
            add_triangle(&mut faces, &mut half_edges, v_curr, v_next, apex_idx);
        }

        // Bottom Cap
        for i in 1..segments - 1 {
            let v0 = bottom_start as u32;
            let v1 = bottom_start as u32 + i + 1;
            let v2 = bottom_start as u32 + i;
            add_triangle(&mut faces, &mut half_edges, v0, v1, v2);
        }

    } else if radius_bottom == 0.0 {
        // Inverted Cone (Tip at bottom)
        // Apex
        let apex_idx = vertices.len() as u32;
        vertices.push(Vertex {
            position: Vec3::new(0.0, 0.0, z_offset),
            first_edge: 0,
        });
        let top_start = create_ring(radius_top, z_offset + height, &mut vertices);

        // Sides (Triangles)
        for i in 0..segments {
            let j = (i + 1) % segments;
            let v_curr = (top_start as u32) + i;
            let v_next = (top_start as u32) + j;

            // Triangle: next -> curr -> apex
            // Winding: Looking from outside. Top ring is CCW.
            // Apex is below.
            // next -> curr is CW on top.
            // We want normal Out.
            // Side normal points XY radial.
            // Top vertices order: curr -> next (CCW).
            // Triangle: curr -> apex -> next ??
            // Let's visualize. Apex (0,0,0). Curr (1,0,1). Next (0,1,1).
            // (1,0,1) -> (0,0,0) -> (0,1,1).
            // V1 = -Curr (-1, 0, -1). V2 = Next (0, 1, 1).
            // Cross: (0 - -1, ..., -1 - 0) -> (1, 1, -1). Points Up/Out?
            // We want normal to point Out.
            // If we use `v_next -> v_curr -> apex`.
            // (0,1,1) -> (1,0,1) -> (0,0,0).
            // V1 = (1,-1,0). V2 = (-1,0,-1).
            // Cross: (1, 1, -1).
            // If we use `v_curr -> v_next -> apex`.
            // (1,0,1) -> (0,1,1) -> (0,0,0).
            // V1 = (-1, 1, 0). V2 = (0, -1, -1).
            // Cross: (-1, -1, 1). Points Down/In?
            // So `next -> curr -> apex` is correct?
            // Wait, standard cone (base bottom): `curr -> next -> apex`.
            // Inverted cone (base top): `apex -> next -> curr`?
            // Let's stick to consistent winding.
            // Top cap is `0 -> i -> i+1`.
            // So `next -> curr -> apex`.
            add_triangle(&mut faces, &mut half_edges, v_next, v_curr, apex_idx);
        }

        // Top Cap
        for i in 1..segments - 1 {
            let v0 = top_start as u32;
            let v1 = top_start as u32 + i;
            let v2 = top_start as u32 + i + 1;
            add_triangle(&mut faces, &mut half_edges, v0, v1, v2);
        }
    }

    stitch_mesh(&mut half_edges)?;

    let mut m = Manifold {
        vertices,
        half_edges,
        faces,
        color: None,
    };

    for (i, edge) in m.half_edges.iter().enumerate() {
        m.vertices[edge.start_vert as usize].first_edge = i as u32;
    }

    Ok(m)
}

#[cfg(test)]
mod tests;
