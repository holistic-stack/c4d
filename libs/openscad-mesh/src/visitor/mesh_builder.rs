//! # Mesh Builder
//!
//! Converts GeometryNode tree to triangle mesh.
//!
//! ## Example
//!
//! ```rust
//! use openscad_eval::GeometryNode;
//! use openscad_mesh::visitor::mesh_builder::build_mesh;
//!
//! let node = GeometryNode::Cube { size: [10.0, 10.0, 10.0], center: false };
//! let mesh = build_mesh(&node).unwrap();
//! ```

use crate::error::MeshError;
use crate::mesh::Mesh;
use openscad_eval::GeometryNode;
use std::f32::consts::PI;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Build a mesh from a geometry node.
///
/// ## Parameters
///
/// - `node`: Root geometry node
///
/// ## Returns
///
/// Triangle mesh
pub fn build_mesh(node: &GeometryNode) -> Result<Mesh, MeshError> {
    let mut builder = MeshBuilder::new();
    builder.build(node)?;
    Ok(builder.mesh)
}

// =============================================================================
// MESH BUILDER
// =============================================================================

/// Mesh builder state.
struct MeshBuilder {
    /// Accumulated mesh.
    mesh: Mesh,
}

impl MeshBuilder {
    /// Create new builder.
    fn new() -> Self {
        Self { mesh: Mesh::new() }
    }

    /// Build mesh from geometry node.
    fn build(&mut self, node: &GeometryNode) -> Result<(), MeshError> {
        match node {
            // 3D Primitives
            GeometryNode::Cube { size, center } => {
                self.build_cube(*size, *center);
            }
            GeometryNode::Sphere { radius, fn_ } => {
                self.build_sphere(*radius, *fn_);
            }
            GeometryNode::Cylinder { height, radius1, radius2, center, fn_ } => {
                self.build_cylinder(*height, *radius1, *radius2, *center, *fn_);
            }

            // Transforms
            GeometryNode::Translate { offset, child } => {
                self.build_translate(*offset, child)?;
            }
            GeometryNode::Rotate { angles, child } => {
                self.build_rotate(*angles, child)?;
            }
            GeometryNode::Scale { factors, child } => {
                self.build_scale(*factors, child)?;
            }
            GeometryNode::Mirror { normal, child } => {
                self.build_mirror(*normal, child)?;
            }
            GeometryNode::Color { child, .. } => {
                // Color is for rendering hints only, pass through
                self.build(child)?;
            }

            // Boolean operations using BSP
            GeometryNode::Union { children } => {
                self.build_union(children)?;
            }
            GeometryNode::Difference { children } => {
                self.build_difference(children)?;
            }
            GeometryNode::Intersection { children } => {
                self.build_intersection(children)?;
            }

            // Groups
            GeometryNode::Group { children } => {
                for child in children {
                    self.build(child)?;
                }
            }

            // Empty
            GeometryNode::Empty => {}

            // Unsupported (2D, extrusions, etc.)
            _ => {
                // Skip unsupported geometries for now
            }
        }

        Ok(())
    }

    // =========================================================================
    // 3D PRIMITIVES
    // =========================================================================

    /// Build cube mesh.
    ///
    /// ## OpenSCAD Cube Spec
    ///
    /// - `size`: [x, y, z] dimensions
    /// - `center`: if false, corner at origin; if true, centered
    fn build_cube(&mut self, size: [f64; 3], center: bool) {
        let [sx, sy, sz] = [size[0] as f32, size[1] as f32, size[2] as f32];
        
        let (min_x, max_x, min_y, max_y, min_z, max_z) = if center {
            (-sx / 2.0, sx / 2.0, -sy / 2.0, sy / 2.0, -sz / 2.0, sz / 2.0)
        } else {
            (0.0, sx, 0.0, sy, 0.0, sz)
        };

        // Each face has 4 vertices and 2 triangles
        // OpenSCAD uses Z-up coordinate system

        // Front face (+Y)
        let v0 = self.mesh.add_vertex(min_x, max_y, min_z, 0.0, 1.0, 0.0);
        let v1 = self.mesh.add_vertex(max_x, max_y, min_z, 0.0, 1.0, 0.0);
        let v2 = self.mesh.add_vertex(max_x, max_y, max_z, 0.0, 1.0, 0.0);
        let v3 = self.mesh.add_vertex(min_x, max_y, max_z, 0.0, 1.0, 0.0);
        self.mesh.add_triangle(v0, v1, v2);
        self.mesh.add_triangle(v0, v2, v3);

        // Back face (-Y)
        let v0 = self.mesh.add_vertex(max_x, min_y, min_z, 0.0, -1.0, 0.0);
        let v1 = self.mesh.add_vertex(min_x, min_y, min_z, 0.0, -1.0, 0.0);
        let v2 = self.mesh.add_vertex(min_x, min_y, max_z, 0.0, -1.0, 0.0);
        let v3 = self.mesh.add_vertex(max_x, min_y, max_z, 0.0, -1.0, 0.0);
        self.mesh.add_triangle(v0, v1, v2);
        self.mesh.add_triangle(v0, v2, v3);

        // Top face (+Z)
        let v0 = self.mesh.add_vertex(min_x, min_y, max_z, 0.0, 0.0, 1.0);
        let v1 = self.mesh.add_vertex(max_x, min_y, max_z, 0.0, 0.0, 1.0);
        let v2 = self.mesh.add_vertex(max_x, max_y, max_z, 0.0, 0.0, 1.0);
        let v3 = self.mesh.add_vertex(min_x, max_y, max_z, 0.0, 0.0, 1.0);
        self.mesh.add_triangle(v0, v1, v2);
        self.mesh.add_triangle(v0, v2, v3);

        // Bottom face (-Z)
        let v0 = self.mesh.add_vertex(min_x, max_y, min_z, 0.0, 0.0, -1.0);
        let v1 = self.mesh.add_vertex(max_x, max_y, min_z, 0.0, 0.0, -1.0);
        let v2 = self.mesh.add_vertex(max_x, min_y, min_z, 0.0, 0.0, -1.0);
        let v3 = self.mesh.add_vertex(min_x, min_y, min_z, 0.0, 0.0, -1.0);
        self.mesh.add_triangle(v0, v1, v2);
        self.mesh.add_triangle(v0, v2, v3);

        // Right face (+X)
        let v0 = self.mesh.add_vertex(max_x, max_y, min_z, 1.0, 0.0, 0.0);
        let v1 = self.mesh.add_vertex(max_x, min_y, min_z, 1.0, 0.0, 0.0);
        let v2 = self.mesh.add_vertex(max_x, min_y, max_z, 1.0, 0.0, 0.0);
        let v3 = self.mesh.add_vertex(max_x, max_y, max_z, 1.0, 0.0, 0.0);
        self.mesh.add_triangle(v0, v1, v2);
        self.mesh.add_triangle(v0, v2, v3);

        // Left face (-X)
        let v0 = self.mesh.add_vertex(min_x, min_y, min_z, -1.0, 0.0, 0.0);
        let v1 = self.mesh.add_vertex(min_x, max_y, min_z, -1.0, 0.0, 0.0);
        let v2 = self.mesh.add_vertex(min_x, max_y, max_z, -1.0, 0.0, 0.0);
        let v3 = self.mesh.add_vertex(min_x, min_y, max_z, -1.0, 0.0, 0.0);
        self.mesh.add_triangle(v0, v1, v2);
        self.mesh.add_triangle(v0, v2, v3);
    }

    /// Build sphere mesh using UV sphere method.
    fn build_sphere(&mut self, radius: f64, fn_: u32) {
        let r = radius as f32;
        let segments = fn_.max(8) as usize;
        let rings = segments / 2;

        // Generate vertices
        let mut vertices: Vec<(f32, f32, f32)> = Vec::new();
        
        for j in 0..=rings {
            let phi = PI * j as f32 / rings as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            for i in 0..=segments {
                let theta = 2.0 * PI * i as f32 / segments as f32;
                let sin_theta = theta.sin();
                let cos_theta = theta.cos();

                let x = r * sin_phi * cos_theta;
                let y = r * sin_phi * sin_theta;
                let z = r * cos_phi;

                vertices.push((x, y, z));
            }
        }

        // Add vertices to mesh
        let vertex_indices: Vec<u32> = vertices.iter()
            .map(|&(x, y, z)| {
                let len = (x * x + y * y + z * z).sqrt();
                let (nx, ny, nz) = if len > 0.0 {
                    (x / len, y / len, z / len)
                } else {
                    (0.0, 0.0, 1.0)
                };
                self.mesh.add_vertex(x, y, z, nx, ny, nz)
            })
            .collect();

        // Generate triangles
        for j in 0..rings {
            for i in 0..segments {
                let i0 = j * (segments + 1) + i;
                let i1 = i0 + 1;
                let i2 = i0 + segments + 1;
                let i3 = i2 + 1;

                // Two triangles per quad (except at poles)
                if j != 0 {
                    self.mesh.add_triangle(
                        vertex_indices[i0],
                        vertex_indices[i2],
                        vertex_indices[i1],
                    );
                }
                if j != rings - 1 {
                    self.mesh.add_triangle(
                        vertex_indices[i1],
                        vertex_indices[i2],
                        vertex_indices[i3],
                    );
                }
            }
        }
    }

    /// Build cylinder mesh.
    fn build_cylinder(&mut self, height: f64, radius1: f64, radius2: f64, center: bool, fn_: u32) {
        let h = height as f32;
        let r1 = radius1 as f32;
        let r2 = radius2 as f32;
        let segments = fn_.max(8) as usize;

        let (z_bottom, z_top) = if center {
            (-h / 2.0, h / 2.0)
        } else {
            (0.0, h)
        };

        // Bottom cap
        let center_bottom = self.mesh.add_vertex(0.0, 0.0, z_bottom, 0.0, 0.0, -1.0);
        let bottom_verts: Vec<u32> = (0..segments)
            .map(|i| {
                let theta = 2.0 * PI * i as f32 / segments as f32;
                let x = r1 * theta.cos();
                let y = r1 * theta.sin();
                self.mesh.add_vertex(x, y, z_bottom, 0.0, 0.0, -1.0)
            })
            .collect();

        for i in 0..segments {
            let next = (i + 1) % segments;
            self.mesh.add_triangle(center_bottom, bottom_verts[next], bottom_verts[i]);
        }

        // Top cap
        let center_top = self.mesh.add_vertex(0.0, 0.0, z_top, 0.0, 0.0, 1.0);
        let top_verts: Vec<u32> = (0..segments)
            .map(|i| {
                let theta = 2.0 * PI * i as f32 / segments as f32;
                let x = r2 * theta.cos();
                let y = r2 * theta.sin();
                self.mesh.add_vertex(x, y, z_top, 0.0, 0.0, 1.0)
            })
            .collect();

        for i in 0..segments {
            let next = (i + 1) % segments;
            self.mesh.add_triangle(center_top, top_verts[i], top_verts[next]);
        }

        // Side faces
        for i in 0..segments {
            let next = (i + 1) % segments;
            let theta = 2.0 * PI * i as f32 / segments as f32;
            let theta_next = 2.0 * PI * next as f32 / segments as f32;

            // Calculate normals (perpendicular to surface)
            let nx1 = theta.cos();
            let ny1 = theta.sin();
            let nx2 = theta_next.cos();
            let ny2 = theta_next.sin();

            let v0 = self.mesh.add_vertex(r1 * nx1, r1 * ny1, z_bottom, nx1, ny1, 0.0);
            let v1 = self.mesh.add_vertex(r1 * nx2, r1 * ny2, z_bottom, nx2, ny2, 0.0);
            let v2 = self.mesh.add_vertex(r2 * nx2, r2 * ny2, z_top, nx2, ny2, 0.0);
            let v3 = self.mesh.add_vertex(r2 * nx1, r2 * ny1, z_top, nx1, ny1, 0.0);

            self.mesh.add_triangle(v0, v1, v2);
            self.mesh.add_triangle(v0, v2, v3);
        }
    }

    // =========================================================================
    // TRANSFORMS
    // =========================================================================

    /// Build translated geometry.
    fn build_translate(&mut self, offset: [f64; 3], child: &GeometryNode) -> Result<(), MeshError> {
        // Build child into temporary mesh
        let mut child_mesh = Mesh::new();
        let mut child_builder = MeshBuilder { mesh: child_mesh };
        child_builder.build(child)?;
        child_mesh = child_builder.mesh;

        // Apply translation directly to vertices (simpler and more efficient)
        let ox = offset[0] as f32;
        let oy = offset[1] as f32;
        let oz = offset[2] as f32;
        
        for i in (0..child_mesh.vertices.len()).step_by(3) {
            child_mesh.vertices[i] += ox;
            child_mesh.vertices[i + 1] += oy;
            child_mesh.vertices[i + 2] += oz;
        }

        // Merge into main mesh
        self.mesh.merge(&child_mesh);
        Ok(())
    }

    /// Build rotated geometry.
    fn build_rotate(&mut self, angles: [f64; 3], child: &GeometryNode) -> Result<(), MeshError> {
        use glam::Mat4;

        let mut child_mesh = Mesh::new();
        let mut child_builder = MeshBuilder { mesh: child_mesh };
        child_builder.build(child)?;
        child_mesh = child_builder.mesh;

        // Convert degrees to radians and create rotation matrix
        let rx = (angles[0] as f32).to_radians();
        let ry = (angles[1] as f32).to_radians();
        let rz = (angles[2] as f32).to_radians();

        let mat = Mat4::from_euler(glam::EulerRot::XYZ, rx, ry, rz);
        let matrix = mat.to_cols_array_2d();
        child_mesh.transform(&matrix);

        self.mesh.merge(&child_mesh);
        Ok(())
    }

    /// Build scaled geometry.
    fn build_scale(&mut self, factors: [f64; 3], child: &GeometryNode) -> Result<(), MeshError> {
        let mut child_mesh = Mesh::new();
        let mut child_builder = MeshBuilder { mesh: child_mesh };
        child_builder.build(child)?;
        child_mesh = child_builder.mesh;

        let matrix = [
            [factors[0] as f32, 0.0, 0.0, 0.0],
            [0.0, factors[1] as f32, 0.0, 0.0],
            [0.0, 0.0, factors[2] as f32, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        child_mesh.transform(&matrix);

        self.mesh.merge(&child_mesh);
        Ok(())
    }

    /// Build mirrored geometry.
    fn build_mirror(&mut self, normal: [f64; 3], child: &GeometryNode) -> Result<(), MeshError> {
        let mut child_mesh = Mesh::new();
        let mut child_builder = MeshBuilder { mesh: child_mesh };
        child_builder.build(child)?;
        child_mesh = child_builder.mesh;

        // Mirror matrix: I - 2 * n * n^T (where n is normalized normal)
        let n = glam::Vec3::new(normal[0] as f32, normal[1] as f32, normal[2] as f32).normalize();
        let matrix = [
            [1.0 - 2.0 * n.x * n.x, -2.0 * n.x * n.y, -2.0 * n.x * n.z, 0.0],
            [-2.0 * n.y * n.x, 1.0 - 2.0 * n.y * n.y, -2.0 * n.y * n.z, 0.0],
            [-2.0 * n.z * n.x, -2.0 * n.z * n.y, 1.0 - 2.0 * n.z * n.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        child_mesh.transform(&matrix);

        self.mesh.merge(&child_mesh);
        Ok(())
    }

    // =========================================================================
    // BOOLEAN OPERATIONS
    // =========================================================================

    /// Build union of children.
    fn build_union(&mut self, children: &[GeometryNode]) -> Result<(), MeshError> {
        if children.is_empty() {
            return Ok(());
        }

        // Build all children into meshes
        let mut meshes: Vec<Mesh> = Vec::new();
        for child in children {
            let mut child_mesh = Mesh::new();
            let mut child_builder = MeshBuilder { mesh: child_mesh };
            child_builder.build(child)?;
            child_mesh = child_builder.mesh;
            if !child_mesh.is_empty() {
                meshes.push(child_mesh);
            }
        }

        if meshes.is_empty() {
            return Ok(());
        }

        // Union all meshes together
        let mut result = meshes.remove(0);
        for mesh in meshes {
            result = crate::ops::boolean::union(&result, &mesh);
        }

        self.mesh.merge(&result);
        Ok(())
    }

    /// Build difference of children (first minus rest).
    fn build_difference(&mut self, children: &[GeometryNode]) -> Result<(), MeshError> {
        if children.is_empty() {
            return Ok(());
        }

        // Build first child
        let mut result = Mesh::new();
        {
            let mut builder = MeshBuilder { mesh: Mesh::new() };
            builder.build(&children[0])?;
            result = builder.mesh;
        }

        if result.is_empty() {
            return Ok(());
        }

        // Subtract remaining children
        for child in &children[1..] {
            let mut child_mesh = Mesh::new();
            let mut child_builder = MeshBuilder { mesh: child_mesh };
            child_builder.build(child)?;
            child_mesh = child_builder.mesh;
            if !child_mesh.is_empty() {
                result = crate::ops::boolean::difference(&result, &child_mesh);
            }
        }

        self.mesh.merge(&result);
        Ok(())
    }

    /// Build intersection of children.
    fn build_intersection(&mut self, children: &[GeometryNode]) -> Result<(), MeshError> {
        if children.is_empty() {
            return Ok(());
        }

        // Build first child
        let mut result = Mesh::new();
        {
            let mut builder = MeshBuilder { mesh: Mesh::new() };
            builder.build(&children[0])?;
            result = builder.mesh;
        }

        if result.is_empty() {
            return Ok(());
        }

        // Intersect with remaining children
        for child in &children[1..] {
            let mut child_mesh = Mesh::new();
            let mut child_builder = MeshBuilder { mesh: child_mesh };
            child_builder.build(child)?;
            child_mesh = child_builder.mesh;
            if !child_mesh.is_empty() {
                result = crate::ops::boolean::intersection(&result, &child_mesh);
            }
        }

        self.mesh.merge(&result);
        Ok(())
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_cube() {
        let node = GeometryNode::Cube {
            size: [10.0, 10.0, 10.0],
            center: false,
        };
        let mesh = build_mesh(&node).unwrap();
        
        assert_eq!(mesh.vertex_count(), 24); // 4 vertices * 6 faces
        assert_eq!(mesh.triangle_count(), 12); // 2 triangles * 6 faces
    }

    #[test]
    fn test_build_cube_centered() {
        let node = GeometryNode::Cube {
            size: [10.0, 10.0, 10.0],
            center: true,
        };
        let mesh = build_mesh(&node).unwrap();
        
        // Check we have both positive and negative coordinates
        let has_neg = mesh.vertices.iter().any(|&v| v < 0.0);
        let has_pos = mesh.vertices.iter().any(|&v| v > 0.0);
        assert!(has_neg && has_pos);
    }

    #[test]
    fn test_build_sphere() {
        let node = GeometryNode::Sphere {
            radius: 5.0,
            fn_: 16,
        };
        let mesh = build_mesh(&node).unwrap();
        
        assert!(!mesh.is_empty());
    }

    #[test]
    fn test_build_translate() {
        let node = GeometryNode::Translate {
            offset: [10.0, 0.0, 0.0],
            child: Box::new(GeometryNode::Cube {
                size: [5.0, 5.0, 5.0],
                center: false,
            }),
        };
        let mesh = build_mesh(&node).unwrap();
        
        // All x values should be >= 10
        for i in (0..mesh.vertices.len()).step_by(3) {
            assert!(mesh.vertices[i] >= 10.0);
        }
    }

    #[test]
    fn test_build_union() {
        let node = GeometryNode::Union {
            children: vec![
                GeometryNode::Cube { size: [5.0, 5.0, 5.0], center: false },
                GeometryNode::Cube { size: [5.0, 5.0, 5.0], center: false },
            ],
        };
        let mesh = build_mesh(&node).unwrap();
        
        // Should have twice the vertices of a single cube
        assert_eq!(mesh.vertex_count(), 48);
    }
}
