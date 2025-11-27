//! # OpenSCAD WASM Interface
//!
//! Browser-safe WebAssembly interface for the OpenSCAD rendering pipeline.
//!
//! ## Architecture
//!
//! ```text
//! JavaScript: web-tree-sitter + tree-sitter-openscad.wasm
//!     ↓ (CST JSON)
//! Rust WASM: render_from_cst()
//!     ↓ (Mesh Data)
//! JavaScript: Three.js WebGL
//! ```
//!
//! ## Browser Safety
//!
//! - NO tree-sitter C dependency (parsing done in JavaScript)
//! - NO WASI or file system access
//! - Pure computation only
//!
//! ## Example (JavaScript)
//!
//! ```javascript
//! import init, { render_from_cst, get_version } from './openscad_wasm.js';
//! import { initParser, parseToJson } from './lib/parser/openscad-parser';
//!
//! await init();
//! await initParser();
//!
//! const cstJson = parseToJson('cube(10);');
//! const result = render_from_cst(cstJson);
//! // result.vertices, result.indices, result.normals are typed arrays
//! ```

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// CONSTANTS
// =============================================================================

/// Current version of the WASM module.
const VERSION: &str = env!("CARGO_PKG_VERSION");

// =============================================================================
// CST TYPES
// =============================================================================
//
// These types mirror the output from web-tree-sitter in JavaScript.
// The CST (Concrete Syntax Tree) is parsed in JS and sent to Rust as JSON.

/// Source position in the parsed code.
///
/// ## Example
///
/// ```json
/// { "row": 0, "column": 5 }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Line number (0-indexed).
    pub row: usize,
    /// Column number (0-indexed).
    pub column: usize,
}

/// CST Node from tree-sitter.
///
/// This structure matches the CstNode interface in openscad-parser.ts.
///
/// ## Example
///
/// ```json
/// {
///   "nodeType": "module_call",
///   "text": "cube(10)",
///   "startByte": 0,
///   "endByte": 8,
///   "startPosition": { "row": 0, "column": 0 },
///   "endPosition": { "row": 0, "column": 8 },
///   "namedChildren": [...],
///   "hasError": false
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CstNode {
    /// Node type from tree-sitter grammar.
    /// Examples: "source_file", "module_call", "number", "identifier"
    pub node_type: String,

    /// Text content of the node.
    pub text: String,

    /// Start byte offset in source.
    pub start_byte: usize,

    /// End byte offset in source.
    pub end_byte: usize,

    /// Start position (row, column).
    pub start_position: Position,

    /// End position (row, column).
    pub end_position: Position,

    /// Named child nodes.
    pub named_children: Vec<CstNode>,

    /// Whether this node has syntax errors.
    pub has_error: bool,
}

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

/// Render from CST JSON (main entry point).
///
/// Accepts CST JSON from web-tree-sitter and returns mesh data.
///
/// ## Parameters
///
/// - `cst_json`: JSON string of CST from tree-sitter
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
///
/// ## Example (JavaScript)
///
/// ```javascript
/// const cstJson = parseToJson('cube(20);');
/// const result = render_from_cst(cstJson);
/// if (result.success) {
///     scene.updateMesh(result.vertices, result.indices, result.normals);
/// }
/// ```
#[wasm_bindgen]
pub fn render_from_cst(cst_json: &str) -> JsValue {
    let start = js_sys::Date::now();

    // Parse CST JSON
    match serde_json::from_str::<CstNode>(cst_json) {
        Ok(cst) => {
            // Log CST info for debugging
            log(&format!(
                "[WASM] CST: type={}, children={}, hasError={}",
                cst.node_type,
                cst.named_children.len(),
                cst.has_error
            ));

            // Extract cube size from CST (proof of concept)
            let size = extract_cube_size(&cst).unwrap_or(10.0);
            log(&format!("[WASM] Extracted cube size: {}", size));

            // Generate cube mesh
            let (vertices, indices, normals) = generate_cube(size, true);
            let render_time_ms = js_sys::Date::now() - start;

            // Return JavaScript object with typed arrays
            create_success_result(vertices, indices, normals, render_time_ms)
        }
        Err(e) => create_error_result(&format!("CST parse error: {}", e)),
    }
}

// =============================================================================
// CST PARSING (Proof of Concept)
// =============================================================================

/// Extract cube size from CST.
///
/// Searches for: source_file > module_call[name="cube"] > arguments > number
///
/// ## Parameters
///
/// - `cst`: Root CST node
///
/// ## Returns
///
/// Cube size if found, None otherwise
fn extract_cube_size(cst: &CstNode) -> Option<f32> {
    for child in &cst.named_children {
        // Look for module_call or transform_call
        if child.node_type == "module_call" || child.node_type == "transform_call" {
            // Check if it's a cube call
            let mut is_cube = false;
            let mut args_node: Option<&CstNode> = None;

            for sub in &child.named_children {
                if sub.node_type == "identifier" && sub.text == "cube" {
                    is_cube = true;
                }
                if sub.node_type == "arguments" {
                    args_node = Some(sub);
                }
            }

            if is_cube {
                if let Some(args) = args_node {
                    return extract_number(args);
                }
            }
        }
        // Recurse into children
        if let Some(size) = extract_cube_size(child) {
            return Some(size);
        }
    }
    None
}

/// Extract a number from an arguments node.
///
/// Recursively searches for number/float/integer nodes.
fn extract_number(node: &CstNode) -> Option<f32> {
    // Direct number types
    if matches!(node.node_type.as_str(), "number" | "float" | "integer") {
        return node.text.parse().ok();
    }

    // Recurse into children
    for child in &node.named_children {
        if let Some(num) = extract_number(child) {
            return Some(num);
        }
    }
    None
}

// =============================================================================
// MESH GENERATION
// =============================================================================

/// Generate a cube mesh.
///
/// Creates vertices, indices, and normals for a cube.
///
/// ## Parameters
///
/// - `size`: Cube size (edge length)
/// - `center`: If true, center at origin; if false, corner at origin
///
/// ## Returns
///
/// Tuple of (vertices, indices, normals) arrays
///
/// ## Example
///
/// ```rust
/// let (vertices, indices, normals) = generate_cube(10.0, true);
/// // vertices: 72 floats (24 vertices * 3 components)
/// // indices: 36 u32s (12 triangles * 3 indices)
/// // normals: 72 floats (24 normals * 3 components)
/// ```
fn generate_cube(size: f32, center: bool) -> (Vec<f32>, Vec<u32>, Vec<f32>) {
    let half = size / 2.0;
    let (min, max) = if center {
        (-half, half)
    } else {
        (0.0, size)
    };

    // 24 vertices (4 per face, for proper normals)
    // Z-up coordinate system (OpenSCAD standard)
    let vertices = vec![
        // Front face (+Y)
        min, max, min, max, max, min, max, max, max, min, max, max,
        // Back face (-Y)
        max, min, min, min, min, min, min, min, max, max, min, max,
        // Top face (+Z)
        min, min, max, max, min, max, max, max, max, min, max, max,
        // Bottom face (-Z)
        min, max, min, max, max, min, max, min, min, min, min, min,
        // Right face (+X)
        max, max, min, max, min, min, max, min, max, max, max, max,
        // Left face (-X)
        min, min, min, min, max, min, min, max, max, min, min, max,
    ];

    // 12 triangles (2 per face)
    let indices: Vec<u32> = vec![
        0, 1, 2, 0, 2, 3,       // Front
        4, 5, 6, 4, 6, 7,       // Back
        8, 9, 10, 8, 10, 11,    // Top
        12, 13, 14, 12, 14, 15, // Bottom
        16, 17, 18, 16, 18, 19, // Right
        20, 21, 22, 20, 22, 23, // Left
    ];

    // Normals per vertex (same for all vertices on a face)
    let normals = vec![
        // Front (+Y)
        0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0,
        // Back (-Y)
        0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0,
        // Top (+Z)
        0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0,
        // Bottom (-Z)
        0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0,
        // Right (+X)
        1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0,
        // Left (-X)
        -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0, -1.0, 0.0, 0.0,
    ];

    (vertices, indices, normals)
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

    /// Test cube mesh generation produces correct sizes.
    #[test]
    fn test_generate_cube_sizes() {
        let (vertices, indices, normals) = generate_cube(10.0, true);

        // 24 vertices * 3 components = 72 floats
        assert_eq!(vertices.len(), 72);
        // 12 triangles * 3 indices = 36
        assert_eq!(indices.len(), 36);
        // 24 normals * 3 components = 72 floats
        assert_eq!(normals.len(), 72);
    }

    /// Test CST number extraction.
    #[test]
    fn test_extract_number() {
        let node = CstNode {
            node_type: "number".to_string(),
            text: "42.5".to_string(),
            start_byte: 0,
            end_byte: 4,
            start_position: Position { row: 0, column: 0 },
            end_position: Position { row: 0, column: 4 },
            named_children: vec![],
            has_error: false,
        };

        assert_eq!(extract_number(&node), Some(42.5));
    }

    /// Test version is not empty.
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
