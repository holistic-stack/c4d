//! Sphere primitive implementation mirroring OpenSCAD's latitude/longitude tessellation.

use std::collections::HashMap;

use crate::core::ds::{Face, HalfEdge, Vertex, NO_INDEX};
use crate::core::vec3::Vec3;
use crate::error::ManifoldError;
use crate::Manifold;

/// Minimum allowed fragment count to avoid degenerate meshes and match OpenSCAD's defaults.
const MIN_SEGMENTS: u32 = 3;

/// Minimum number of latitude rings (OpenSCAD uses `(fragments + 1) / 2`).
const MIN_RINGS: u32 = 2;

/// Sphere primitive generator configured via radius and OpenSCAD-style fragment count.
///
/// # Examples
/// ```
/// use manifold_rs::primitives::sphere::Sphere;
///
/// let sphere = Sphere::new(10.0, 24).expect("valid sphere configuration");
/// let manifold = sphere.to_manifold().expect("mesh is produced");
/// assert!(manifold.validate().is_ok());
/// ```
#[derive(Debug)]
pub struct Sphere {
    radius: f64,
    fragments: u32,
    rings: u32,
}

impl Sphere {
    /// Creates a new sphere configuration validated against topology constraints.
    ///
    /// # Examples
    /// ```
    /// use manifold_rs::primitives::sphere::Sphere;
    ///
    /// let sphere = Sphere::new(5.0, 0).unwrap();
    /// assert_eq!(sphere.fragments(), 3); // segments clamped to minimum
    /// ```
    pub fn new(radius: f64, segments: u32) -> Result<Self, ManifoldError> {
        if radius <= 0.0 {
            return Err(ManifoldError::InvalidTopology(
                "sphere radius must be positive".to_string(),
            ));
        }
        let fragments = segments.max(MIN_SEGMENTS);
        let rings = ((fragments + 1) / 2).max(MIN_RINGS);

        Ok(Self {
            radius,
            fragments,
            rings,
        })
    }

    /// Returns the number of longitudinal fragments used by this configuration.
    ///
    /// # Examples
    /// ```text
    /// Sphere::new(2.0, 12) -> fragments() == 12
    /// ```
    pub fn fragments(&self) -> u32 {
        self.fragments
    }

    /// Converts the configuration into a manifold mesh using OpenSCAD's tessellation rules.
    ///
    /// # Examples
    /// ```text
    /// Sphere::new(2.0, 24).to_manifold() -> vertex_count > 0
    /// ```
    pub fn to_manifold(&self) -> Result<Manifold, ManifoldError> {
        let vertices = self.generate_vertices();
        let faces = self.generate_faces();
        self.build_half_edge(&vertices, &faces)
    }

    /// Generates all latitude/longitude vertices (matching OpenSCAD's `generate_circle`).
    ///
    /// # Examples
    /// ```text
    /// (ring=0, fragment=0) -> (x=r*sin(phi), y=0, z=r*cos(phi))
    /// ```
    fn generate_vertices(&self) -> Vec<Vec3> {
        let fragments = self.fragments as usize;
        let rings = self.rings as usize;
        let mut vertices = Vec::with_capacity(fragments * rings);
        let rings_f64 = self.rings as f64;
        let fragments_f64 = self.fragments as f64;

        for ring in 0..rings {
            let phi_deg = 180.0 * (ring as f64 + 0.5) / rings_f64;
            let phi_rad = phi_deg.to_radians();
            let ring_radius = self.radius * phi_rad.sin();
            let z = self.radius * phi_rad.cos();

            for fragment in 0..fragments {
                let theta_deg = 360.0 * fragment as f64 / fragments_f64;
                let theta_rad = theta_deg.to_radians();
                let x = ring_radius * theta_rad.cos();
                let y = ring_radius * theta_rad.sin();
                vertices.push(Vec3::new(x, y, z));
            }
        }

        vertices
    }

    /// Generates triangle indices equivalent to OpenSCAD's PolySet topology.
    ///
    /// # Examples
    /// ```text
    /// fragments=8, rings=4 => faces.len() = (3 * 8 * 2) + (8 - 2) * 2 = 56
    /// ```
    fn generate_faces(&self) -> Vec<[u32; 3]> {
        let fragments = self.fragments as usize;
        let rings = self.rings as usize;
        let middle_triangles = if rings > 1 {
            (rings - 1) * fragments * 2
        } else {
            0
        };
        let cap_triangles = if fragments > 2 {
            (fragments - 2) * 2
        } else {
            0
        };
        let mut faces = Vec::with_capacity(middle_triangles + cap_triangles);

        self.add_top_cap_faces(&mut faces);
        self.add_middle_band_faces(&mut faces);
        self.add_bottom_cap_faces(&mut faces);

        faces
    }

    /// Adds the top fan triangles that approximate OpenSCAD's polar cap.
    ///
    /// # Examples
    /// ```text
    /// fragments=4 => faces.push([0,1,2]) + [0,2,3]
    /// ```
    fn add_top_cap_faces(&self, faces: &mut Vec<[u32; 3]>) {
        let fragments = self.fragments as usize;
        if fragments < 3 {
            return;
        }

        for i in 1..fragments - 1 {
            faces.push([
                0,
                i as u32,
                (i + 1) as u32,
            ]);
        }
    }

    /// Adds the rectangular band triangles between each latitude pair.
    ///
    /// # Examples
    /// ```text
    /// fragments=4 -> ring quad indices expanded into two triangles per cell
    /// ```
    fn add_middle_band_faces(&self, faces: &mut Vec<[u32; 3]>) {
        let fragments = self.fragments as usize;
        let rings = self.rings as usize;

        for ring in 0..(rings - 1) {
            let ring_start = ring * fragments;
            let next_start = (ring + 1) * fragments;
            for fragment in 0..fragments {
                let next_fragment = (fragment + 1) % fragments;
                let v0 = (ring_start + next_fragment) as u32;
                let v1 = (ring_start + fragment) as u32;
                let v2 = (next_start + fragment) as u32;
                let v3 = (next_start + next_fragment) as u32;

                faces.push([v0, v1, v2]);
                faces.push([v0, v2, v3]);
            }
        }
    }

    /// Adds the bottom fan triangles matching OpenSCAD's reversed index order.
    ///
    /// # Examples
    /// ```text
    /// fragments=5 => faces.push([base, base-1, base-2]) ...
    /// ```
    fn add_bottom_cap_faces(&self, faces: &mut Vec<[u32; 3]>) {
        let fragments = self.fragments as usize;
        if fragments < 3 {
            return;
        }
        let rings = self.rings as usize;
        let last_ring_start = (rings - 1) * fragments;
        let base = (last_ring_start + fragments - 1) as u32;

        for i in 1..fragments - 1 {
            let v1 = base - i as u32;
            let v2 = base - (i as u32 + 1);
            faces.push([base, v1, v2]);
        }
    }

    /// Builds the half-edge manifold from the generated vertices/faces.
    ///
    /// # Examples
    /// ```text
    /// Called internally after tessellation to populate Manifold buffers
    /// ```
    fn build_half_edge(
        &self,
        vertices: &[Vec3],
        faces: &[[u32; 3]],
    ) -> Result<Manifold, ManifoldError> {
        let mut manifold = Manifold::new();
        manifold.vertices = vertices
            .iter()
            .map(|pos| Vertex::new(*pos, NO_INDEX))
            .collect();

        let mut edge_map = HashMap::new();

        for (face_idx, triangle) in faces.iter().enumerate() {
            let [v0, v1, v2] = *triangle;
            let edge_base = manifold.half_edges.len() as u32;

            let normal = {
                let a = vertices[v0 as usize];
                let b = vertices[v1 as usize];
                let c = vertices[v2 as usize];
                (b - a).cross(c - a).normalize()
            };

            manifold.half_edges.push(HalfEdge::new(v0, v1, edge_base + 1, NO_INDEX, face_idx as u32));
            manifold.half_edges.push(HalfEdge::new(v1, v2, edge_base + 2, NO_INDEX, face_idx as u32));
            manifold.half_edges.push(HalfEdge::new(v2, v0, edge_base, NO_INDEX, face_idx as u32));

            manifold.faces.push(Face::new(edge_base, normal));

            for i in 0..3 {
                let start = triangle[i];
                let end = triangle[(i + 1) % 3];
                let edge_idx = edge_base + i as u32;
                edge_map.insert((start, end), edge_idx);

                let vertex = &mut manifold.vertices[start as usize];
                if vertex.first_edge == NO_INDEX {
                    vertex.first_edge = edge_idx;
                }
            }
        }

        for edge in &mut manifold.half_edges {
            let key = (edge.end_vert, edge.start_vert);
            edge.pair_edge = *edge_map
                .get(&key)
                .ok_or_else(|| ManifoldError::InvalidTopology("sphere mesh has open edges".into()))?;
        }

        Ok(manifold)
    }
}

#[cfg(test)]
mod tests;
