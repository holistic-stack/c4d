//! # OpenSCAD WASM Interface
//!
//! Browser-safe WebAssembly interface for the OpenSCAD rendering pipeline.
//!
//! ## Architecture
//!
//! ```text
//! JavaScript: render(source)
//!     ↓
//! Rust WASM: Full pipeline (parser → AST → eval → mesh)
//!     ↓
//! JavaScript: Three.js WebGL
//! ```
//!
//! ## Browser Safety
//!
//! - Pure Rust parser (no C dependencies)
//! - NO WASI or file system access
//! - Pure computation only
//!
//! ## Example (JavaScript)
//!
//! ```javascript
//! import init, { render } from './openscad_wasm.js';
//!
//! await init();
//!
//! const result = render('cube(10);');
//! // result.vertices, result.indices, result.normals are typed arrays
//! ```

use wasm_bindgen::prelude::*;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Current version of the WASM module.
const VERSION: &str = env!("CARGO_PKG_VERSION");

// =============================================================================
// INITIALIZATION
// =============================================================================

/// Initialize the WASM module.
///
/// Sets up panic hook for better error messages in browser console.
/// Call this once before using any other functions.
///
/// ## Example (JavaScript)
///
/// ```javascript
/// import init from './openscad_wasm.js';
/// await init();
/// ```
#[wasm_bindgen(start)]
pub fn wasm_init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// =============================================================================
// PUBLIC API
// =============================================================================

/// Get the WASM module version.
///
/// ## Returns
///
/// Version string (e.g., "0.1.0")
///
/// ## Example (JavaScript)
///
/// ```javascript
/// const version = get_version();
/// console.log(`WASM version: ${version}`);
/// ```
#[wasm_bindgen]
pub fn get_version() -> String {
    VERSION.to_string()
}

/// Render OpenSCAD source code to mesh (main entry point).
///
/// Full pipeline: parser → AST → evaluator → mesh generator.
/// All processing done in pure Rust - no external dependencies.
///
/// ## Parameters
///
/// - `source`: OpenSCAD source code string
///
/// ## Returns
///
/// JavaScript object with typed arrays:
/// - `success`: boolean
/// - `vertices`: Float32Array (x, y, z positions)
/// - `indices`: Uint32Array (triangle indices)
/// - `normals`: Float32Array (x, y, z normals)
/// - `vertexCount`: number
/// - `triangleCount`: number
/// - `renderTimeMs`: number
/// - `error`: string (only if success is false)
///
/// ## Example (JavaScript)
///
/// ```javascript
/// const result = render('cube(10);');
/// if (result.success) {
///     scene.updateMesh(result.vertices, result.indices, result.normals);
/// } else {
///     console.error(result.error);
/// }
/// ```
#[wasm_bindgen]
pub fn render(source: &str) -> JsValue {
    let start = js_sys::Date::now();

    // Full pipeline: source → mesh
    match manifold_rs::render(source) {
        Ok(mesh) => {
            let render_time_ms = js_sys::Date::now() - start;
            create_success_result(mesh.vertices, mesh.indices, mesh.normals, render_time_ms)
        }
        Err(e) => create_error_result(&format!("Render error: {}", e)),
    }
}

// =============================================================================
// RESULT HELPERS
// =============================================================================

/// Create a success result with mesh data.
fn create_success_result(
    vertices: Vec<f32>,
    indices: Vec<u32>,
    normals: Vec<f32>,
    render_time_ms: f64,
) -> JsValue {
    let result = js_sys::Object::new();

    // Create typed arrays (zero-copy transfer)
    let vertices_array = js_sys::Float32Array::from(vertices.as_slice());
    let indices_array = js_sys::Uint32Array::from(indices.as_slice());
    let normals_array = js_sys::Float32Array::from(normals.as_slice());

    let vertex_count = (vertices.len() / 3) as u32;
    let triangle_count = (indices.len() / 3) as u32;

    // Set result properties
    let _ = js_sys::Reflect::set(&result, &"success".into(), &true.into());
    let _ = js_sys::Reflect::set(&result, &"vertices".into(), &vertices_array);
    let _ = js_sys::Reflect::set(&result, &"indices".into(), &indices_array);
    let _ = js_sys::Reflect::set(&result, &"normals".into(), &normals_array);
    let _ = js_sys::Reflect::set(&result, &"vertexCount".into(), &vertex_count.into());
    let _ = js_sys::Reflect::set(&result, &"triangleCount".into(), &triangle_count.into());
    let _ = js_sys::Reflect::set(&result, &"renderTimeMs".into(), &render_time_ms.into());

    result.into()
}

/// Create an error result.
fn create_error_result(error: &str) -> JsValue {
    let result = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&result, &"success".into(), &false.into());
    let _ = js_sys::Reflect::set(&result, &"error".into(), &error.into());
    result.into()
}

// =============================================================================
// LOGGING
// =============================================================================

/// Log to browser console.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test rendering cube produces mesh data.
    #[test]
    fn test_render_cube() {
        let mesh = manifold_rs::render("cube(10);").unwrap();
        
        // 24 vertices * 3 components = 72 floats
        assert_eq!(mesh.vertices.len(), 72);
        // 12 triangles * 3 indices = 36
        assert_eq!(mesh.indices.len(), 36);
        // 24 normals * 3 components = 72 floats
        assert_eq!(mesh.normals.len(), 72);
    }

    /// Test rendering sphere produces mesh data.
    #[test]
    fn test_render_sphere() {
        let mesh = manifold_rs::render("sphere(5);").unwrap();
        
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    /// Test version is not empty.
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
