//! # Mesh Handle
//!
//! WASM-friendly wrapper for mesh data that can be transferred to JavaScript.

use wasm_bindgen::prelude::*;

/// A handle to mesh data that can be accessed from JavaScript.
///
/// Provides zero-copy access to vertex and index buffers via typed arrays.
///
/// # Example (JavaScript)
///
/// ```javascript
/// const mesh = compile_and_render("cube(10);");
///
/// // Get counts
/// const vertexCount = mesh.vertex_count();
/// const triangleCount = mesh.triangle_count();
///
/// // Get buffers for Three.js
/// const vertices = mesh.vertices();  // Float32Array
/// const indices = mesh.indices();    // Uint32Array
///
/// // Create BufferGeometry
/// const geometry = new THREE.BufferGeometry();
/// geometry.setAttribute('position', new THREE.BufferAttribute(vertices, 3));
/// geometry.setIndex(new THREE.BufferAttribute(indices, 1));
/// ```
#[wasm_bindgen]
pub struct MeshHandle {
    /// Vertex positions as [x, y, z, x, y, z, ...]
    vertices: Vec<f32>,
    /// Triangle indices as [i0, i1, i2, i0, i1, i2, ...]
    indices: Vec<u32>,
    /// Optional vertex normals as [nx, ny, nz, ...]
    normals: Option<Vec<f32>>,
    /// Optional vertex colors as [r, g, b, a, ...]
    colors: Option<Vec<f32>>,
    /// Number of vertices
    vertex_count: u32,
    /// Number of triangles
    triangle_count: u32,
}

#[wasm_bindgen]
impl MeshHandle {
    /// Returns the number of vertices.
    #[wasm_bindgen(getter)]
    pub fn vertex_count(&self) -> u32 {
        self.vertex_count
    }

    /// Returns the number of triangles.
    #[wasm_bindgen(getter)]
    pub fn triangle_count(&self) -> u32 {
        self.triangle_count
    }

    /// Returns the vertex positions as a Float32Array.
    ///
    /// Format: [x, y, z, x, y, z, ...]
    /// Length: vertex_count * 3
    #[wasm_bindgen]
    pub fn vertices(&self) -> js_sys::Float32Array {
        js_sys::Float32Array::from(&self.vertices[..])
    }

    /// Returns the triangle indices as a Uint32Array.
    ///
    /// Format: [i0, i1, i2, i0, i1, i2, ...]
    /// Length: triangle_count * 3
    #[wasm_bindgen]
    pub fn indices(&self) -> js_sys::Uint32Array {
        js_sys::Uint32Array::from(&self.indices[..])
    }

    /// Returns the vertex normals as a Float32Array, if available.
    ///
    /// Format: [nx, ny, nz, nx, ny, nz, ...]
    /// Length: vertex_count * 3
    #[wasm_bindgen]
    pub fn normals(&self) -> Option<js_sys::Float32Array> {
        self.normals
            .as_ref()
            .map(|n| js_sys::Float32Array::from(&n[..]))
    }

    /// Returns the vertex colors as a Float32Array, if available.
    ///
    /// Format: [r, g, b, a, r, g, b, a, ...]
    /// Length: vertex_count * 4
    #[wasm_bindgen]
    pub fn colors(&self) -> Option<js_sys::Float32Array> {
        self.colors
            .as_ref()
            .map(|c| js_sys::Float32Array::from(&c[..]))
    }

    /// Returns true if the mesh has normals.
    #[wasm_bindgen]
    pub fn has_normals(&self) -> bool {
        self.normals.is_some()
    }

    /// Returns true if the mesh has colors.
    #[wasm_bindgen]
    pub fn has_colors(&self) -> bool {
        self.colors.is_some()
    }

    /// Returns true if the mesh is empty.
    #[wasm_bindgen]
    pub fn is_empty(&self) -> bool {
        self.vertex_count == 0
    }
}

impl MeshHandle {
    /// Creates a MeshHandle from a Mesh.
    pub fn from_mesh(mesh: openscad_mesh::Mesh) -> Self {
        let vertex_count = mesh.vertex_count() as u32;
        let triangle_count = mesh.triangle_count() as u32;

        let vertices = mesh.vertices_f32();
        let indices = mesh.indices_u32();
        let normals = mesh.normals_f32();

        // Convert colors if present
        let colors = mesh.colors().map(|c| {
            let mut result = Vec::with_capacity(c.len() * 4);
            for color in c {
                result.extend_from_slice(color);
            }
            result
        });

        Self {
            vertices,
            indices,
            normals,
            colors,
            vertex_count,
            triangle_count,
        }
    }
}
