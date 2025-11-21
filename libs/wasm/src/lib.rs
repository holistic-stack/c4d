//! WASM-facing entry points for the Rust OpenSCAD pipeline.
//!
//! This crate is compiled to a `cdylib` and consumed from JavaScript via
//! `wasm-bindgen`. Native tests interact with the internal helper
//! `compile_and_count_nodes_internal` to avoid depending on a JS host.
//!
//! ```
//! let count = wasm::compile_and_count_nodes_internal("cube(1);").unwrap();
//! assert_eq!(count, 1);
//! ```

use config::constants::DEFAULT_SEGMENTS;
use openscad_eval::{Evaluator, InMemoryFilesystem};
use wasm_bindgen::prelude::*;

/// Installs a panic hook that forwards Rust panics to the browser console.
///
/// # Examples
/// ```no_run
/// // In JavaScript: import and call once at startup.
/// // import { init_panic_hook } from "wasm";
/// // init_panic_hook();
/// ```
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Returns the default tessellation segment count used by the geometry
/// pipeline. This is currently a thin wrapper around a shared constant.
///
/// # Examples
/// ```
/// let segments = wasm::default_segments();
/// assert!(segments >= 3);
/// ```
#[wasm_bindgen]
pub fn default_segments() -> u32 {
    DEFAULT_SEGMENTS
}

/// Compiles OpenSCAD source and returns the number of geometry nodes
/// produced by the current evaluator pipeline.
///
/// This function is the primary entry point used from JavaScript. For Rust
/// tests, prefer `compile_and_count_nodes_internal`, which exposes Rust
/// error types directly.
///
/// # Errors
/// Returns a JavaScript error value containing a human-readable message
/// when evaluation fails.
///
/// # Examples
/// ```no_run
/// // In JavaScript: await compile_and_count_nodes("cube(1);");
/// ```
#[wasm_bindgen]
pub fn compile_and_count_nodes(source: &str) -> Result<u32, JsValue> {
    compile_and_count_nodes_internal(source)
        .map(|count| u32::try_from(count).unwrap_or(u32::MAX))
        .map_err(|err| JsValue::from_str(&err.to_string()))
}

/// Host-only helper that evaluates OpenSCAD source and returns the number of
/// generated geometry nodes.
///
/// # Examples
/// ```
/// let count = wasm::compile_and_count_nodes_internal("cube(1);").unwrap();
/// assert_eq!(count, 1);
/// ```
pub fn compile_and_count_nodes_internal(
    source: &str,
) -> Result<usize, openscad_eval::EvaluationError> {
    let filesystem = InMemoryFilesystem::default();
    let evaluator = Evaluator::new(filesystem);
    let nodes = evaluator.evaluate_source(source)?;
    Ok(nodes.len())
}

mod diagnostics;

pub use diagnostics::{Diagnostic, DiagnosticList, Severity};

/// Mesh handle returned from compilation.
///
/// Contains vertex and index counts for the rendered mesh.
///
/// # Examples
/// ```no_run
/// // In JavaScript:
/// // const result = await compile_and_render("cube([1, 1, 1]);");
/// // console.log(result.vertex_count(), result.triangle_count());
/// ```
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct MeshHandle {
    vertex_count: usize,
    triangle_count: usize,
    vertices: Vec<f32>,
    indices: Vec<u32>,
}

#[wasm_bindgen]
impl MeshHandle {
    /// Returns the number of vertices in the mesh.
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    /// Returns the number of triangles in the mesh.
    pub fn triangle_count(&self) -> usize {
        self.triangle_count
    }

    /// Returns the vertex buffer as a Float32Array.
    pub fn vertices(&self) -> Vec<f32> {
        self.vertices.clone()
    }

    /// Returns the index buffer as a Uint32Array.
    pub fn indices(&self) -> Vec<u32> {
        self.indices.clone()
    }
}

/// Compiles OpenSCAD source and renders it to a mesh.
///
/// This is the main entry point for the pipeline. It parses the source,
/// evaluates it, and generates a mesh suitable for GPU rendering.
///
/// # Errors
/// Returns a JavaScript error containing diagnostics if compilation fails.
///
/// # Examples
/// ```no_run
/// // In JavaScript:
/// // try {
/// //   const mesh = await compile_and_render("cube([2, 2, 2]);");
/// //   console.log("Vertices:", mesh.vertex_count());
/// // } catch (error) {
/// //   console.error("Compilation failed:", error);
/// // }
/// ```
#[wasm_bindgen]
pub fn compile_and_render(source: &str) -> Result<MeshHandle, JsValue> {
    compile_and_render_internal(source)
        .map_err(|diagnostics| {
            // Convert diagnostics to a JS-friendly error message
            let messages: Vec<String> = diagnostics
                .iter()
                .map(|d| format!("{}", d))
                .collect();
            JsValue::from_str(&messages.join("\n"))
        })
}

/// Internal implementation of compile_and_render.
///
/// Returns diagnostics on error for better error reporting.
pub fn compile_and_render_internal(
    source: &str,
) -> Result<MeshHandle, Vec<openscad_ast::Diagnostic>> {
    let buffers = manifold_rs::from_source(source)?;

    Ok(MeshHandle {
        vertex_count: buffers.vertex_count(),
        triangle_count: buffers.triangle_count(),
        vertices: buffers.vertices,
        indices: buffers.indices,
    })
}

#[cfg(test)]
mod tests;
