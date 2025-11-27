//! # OpenSCAD WASM
//!
//! WebAssembly bindings for the OpenSCAD pipeline.
//! This crate provides the browser-facing API for compiling OpenSCAD
//! source code to 3D meshes.
//!
//! ## Architecture
//!
//! The browser pipeline uses web-tree-sitter for parsing:
//!
//! ```text
//! OpenSCAD Source → web-tree-sitter (JS) → Serialized CST (JSON)
//! Serialized CST → render_from_cst (Rust) → Mesh → Three.js
//! ```
//!
//! ## Usage (JavaScript)
//!
//! ### Browser-Safe API (Recommended)
//!
//! ```javascript
//! import init, { render_from_cst } from './openscad-wasm';
//! import { initParser, parseOpenSCAD, serializeTree } from './parser/openscad-parser';
//!
//! await init();
//! await initParser();
//!
//! const { tree } = parseOpenSCAD("cube(10);");
//! const cst = serializeTree(tree);
//! const mesh = render_from_cst(JSON.stringify(cst));
//! ```
//!
//! ### Native API (CLI/Server only)
//!
//! ```javascript
//! import init, { compile_and_render } from './openscad-wasm';
//!
//! await init();
//! const mesh = compile_and_render("cube(10);");
//! ```

use wasm_bindgen::prelude::*;

pub mod diagnostics;
pub mod mesh_handle;

pub use diagnostics::Diagnostic;
pub use mesh_handle::MeshHandle;

/// Initializes the WASM module.
///
/// Call this once before using any other functions.
/// Sets up panic hooks for better error messages in debug builds.
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// Note: compile_and_render and compile_and_count_nodes are not available
// in WASM builds because they require the native tree-sitter parser.
// Use render_from_cst instead, which accepts a pre-parsed CST from
// web-tree-sitter.

/// Renders a mesh from a serialized CST (browser-safe).
///
/// This function accepts a JSON-serialized CST from web-tree-sitter
/// and returns a mesh handle. This is the recommended API for browser use
/// as it avoids the C runtime dependencies of the native tree-sitter parser.
///
/// # Arguments
///
/// * `cst_json` - JSON string containing the serialized CST from web-tree-sitter
///
/// # Returns
///
/// A `MeshHandle` containing vertex and index buffers for rendering.
///
/// # Errors
///
/// Throws a JavaScript error with a `diagnostics` property if:
/// - The JSON is invalid
/// - The CST contains syntax errors
/// - Evaluation fails
/// - Mesh generation fails
///
/// # Example (JavaScript)
///
/// ```javascript
/// import { initParser, parseOpenSCAD, serializeTree } from './parser/openscad-parser';
/// import init, { render_from_cst } from './openscad-wasm';
///
/// await init();
/// await initParser();
///
/// try {
///     const { tree, errors } = parseOpenSCAD("cube(10);");
///     if (errors.length > 0) {
///         console.error("Syntax errors:", errors);
///         return;
///     }
///     const cst = serializeTree(tree);
///     const mesh = render_from_cst(JSON.stringify(cst));
///     console.log(`Vertices: ${mesh.vertex_count()}`);
/// } catch (error) {
///     console.error("Render error:", error);
/// }
/// ```
#[wasm_bindgen]
pub fn render_from_cst(cst_json: &str) -> Result<MeshHandle, JsValue> {
    // Parse the JSON into a SerializedNode
    let cst: openscad_ast::SerializedNode = serde_json::from_str(cst_json)
        .map_err(|e| {
            let diagnostic = diagnostics::Diagnostic::error(
                format!("Invalid CST JSON: {}", e),
                0,
                0,
            );
            diagnostics::build_error_payload(vec![diagnostic])
        })?;

    // Performance timing
    let start_time = js_sys::Date::now();

    // Parse CST to statements
    let statements = openscad_ast::parse_from_cst(&cst)
        .map_err(|e| {
            let diagnostic = diagnostics::Diagnostic::error(
                format!("Parse error: {:?}", e),
                0,
                0,
            );
            diagnostics::build_error_payload(vec![diagnostic])
        })?;
    let parse_time = js_sys::Date::now() - start_time;

    // Evaluate statements
    let eval_start = js_sys::Date::now();
    let mut ctx = openscad_eval::EvaluationContext::new();
    let geometry = openscad_eval::evaluator::evaluate_statements(&statements, &mut ctx)
        .map_err(|e| {
            let diagnostic = diagnostics::from_eval_error(&e);
            diagnostics::build_error_payload(vec![diagnostic])
        })?;
    let eval_time = js_sys::Date::now() - eval_start;

    // Convert geometry to mesh
    let mesh_start = js_sys::Date::now();
    let mesh = openscad_mesh::from_ir::geometry_to_mesh(&geometry)
        .map_err(|e| {
            let diagnostic = diagnostics::from_mesh_error(&e);
            diagnostics::build_error_payload(vec![diagnostic])
        })?;
    let mesh_time = js_sys::Date::now() - mesh_start;

    // Log performance breakdown
    web_sys::console::log_1(&format!(
        "[WASM] Performance: parse={:.0}ms, eval={:.0}ms, mesh={:.0}ms, total={:.0}ms",
        parse_time, eval_time, mesh_time, js_sys::Date::now() - start_time
    ).into());

    Ok(MeshHandle::from_mesh(mesh))
}

// Tests are in libs/openscad-mesh which has the native-parser feature enabled
