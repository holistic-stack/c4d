use crate::Vec3;

// Common triangulation logic for primitives.

/// Tesselates a polygon defined by an outer loop and optional hole loops.
/// For the MVP, this implements a simple ear-clipping or fan triangulation for convex/simple polygons.
/// Complex polygons with holes will require a robust triangulator (e.g., earcutr).
pub fn manifold_from_contours(loops: Vec<Vec<Vec3>>) -> Result<crate::Manifold, String> {
    if loops.is_empty() {
        return Ok(crate::Manifold::new());
    }

    let mut buffers = crate::MeshBuffers::new();
    let mut vertex_offset = 0;

    for loop_pts in loops {
        if loop_pts.len() < 3 { continue; } // Skip degenerate loops

        // Add vertices
        for p in &loop_pts {
            buffers.vertices.push(p.x as f32);
            buffers.vertices.push(p.y as f32);
            buffers.vertices.push(p.z as f32);
        }

        // Triangulate this loop (Fan - assumes convex/simple)
        // TODO: Handle holes (negative loops) correctly using earcut.
        // Currently, we treat every loop as a solid positive polygon.

        // Front faces (CCW)
        for i in 1..loop_pts.len() - 1 {
            buffers.indices.push(vertex_offset);
            buffers.indices.push(vertex_offset + i as u32);
            buffers.indices.push(vertex_offset + (i + 1) as u32);
        }

        // Note: We do NOT generate back faces. Generating back faces with shared vertices
        // creates non-manifold topology (edges shared by >2 faces or duplicated directed edges)
        // which breaks the half-edge structure.
        // 2D shapes are represented as single-sided sheets.

        vertex_offset += loop_pts.len() as u32;
    }

    crate::Manifold::from_mesh_buffers(buffers).map_err(|e| e.to_string())
}
