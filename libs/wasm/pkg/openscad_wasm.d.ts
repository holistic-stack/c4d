/* tslint:disable */
/* eslint-disable */
/**
 * Renders a mesh from a serialized CST (browser-safe).
 *
 * This function accepts a JSON-serialized CST from web-tree-sitter
 * and returns a mesh handle. This is the recommended API for browser use
 * as it avoids the C runtime dependencies of the native tree-sitter parser.
 *
 * # Arguments
 *
 * * `cst_json` - JSON string containing the serialized CST from web-tree-sitter
 *
 * # Returns
 *
 * A `MeshHandle` containing vertex and index buffers for rendering.
 *
 * # Errors
 *
 * Throws a JavaScript error with a `diagnostics` property if:
 * - The JSON is invalid
 * - The CST contains syntax errors
 * - Evaluation fails
 * - Mesh generation fails
 *
 * # Example (JavaScript)
 *
 * ```javascript
 * import { initParser, parseOpenSCAD, serializeTree } from './parser/openscad-parser';
 * import init, { render_from_cst } from './openscad-wasm';
 *
 * await init();
 * await initParser();
 *
 * try {
 *     const { tree, errors } = parseOpenSCAD("cube(10);");
 *     if (errors.length > 0) {
 *         console.error("Syntax errors:", errors);
 *         return;
 *     }
 *     const cst = serializeTree(tree);
 *     const mesh = render_from_cst(JSON.stringify(cst));
 *     console.log(`Vertices: ${mesh.vertex_count()}`);
 * } catch (error) {
 *     console.error("Render error:", error);
 * }
 * ```
 */
export function render_from_cst(cst_json: string): MeshHandle;
/**
 * Initializes the WASM module.
 *
 * Call this once before using any other functions.
 * Sets up panic hooks for better error messages in debug builds.
 */
export function init(): void;
/**
 * A diagnostic message that can be accessed from JavaScript.
 *
 * # Example (JavaScript)
 *
 * ```javascript
 * try {
 *     compile_and_render("invalid code");
 * } catch (error) {
 *     for (const diag of error.diagnostics) {
 *         console.error(`${diag.severity}: ${diag.message}`);
 *         console.error(`  at ${diag.start}..${diag.end}`);
 *         if (diag.hint) {
 *             console.error(`  hint: ${diag.hint}`);
 *         }
 *     }
 * }
 * ```
 */
export class Diagnostic {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Returns the end byte offset in the source.
   */
  readonly end: number;
  /**
   * Returns the optional hint for fixing the issue.
   */
  readonly hint: string | undefined;
  /**
   * Returns the start byte offset in the source.
   */
  readonly start: number;
  /**
   * Returns the diagnostic message.
   */
  readonly message: string;
  /**
   * Returns the severity level ("error" or "warning").
   */
  readonly severity: string;
}
/**
 * A handle to mesh data that can be accessed from JavaScript.
 *
 * Provides zero-copy access to vertex and index buffers via typed arrays.
 *
 * # Example (JavaScript)
 *
 * ```javascript
 * const mesh = compile_and_render("cube(10);");
 *
 * // Get counts
 * const vertexCount = mesh.vertex_count();
 * const triangleCount = mesh.triangle_count();
 *
 * // Get buffers for Three.js
 * const vertices = mesh.vertices();  // Float32Array
 * const indices = mesh.indices();    // Uint32Array
 *
 * // Create BufferGeometry
 * const geometry = new THREE.BufferGeometry();
 * geometry.setAttribute('position', new THREE.BufferAttribute(vertices, 3));
 * geometry.setIndex(new THREE.BufferAttribute(indices, 1));
 * ```
 */
export class MeshHandle {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Returns true if the mesh has colors.
   */
  has_colors(): boolean;
  /**
   * Returns true if the mesh has normals.
   */
  has_normals(): boolean;
  /**
   * Returns the vertex colors as a Float32Array, if available.
   *
   * Format: [r, g, b, a, r, g, b, a, ...]
   * Length: vertex_count * 4
   */
  colors(): Float32Array | undefined;
  /**
   * Returns the triangle indices as a Uint32Array.
   *
   * Format: [i0, i1, i2, i0, i1, i2, ...]
   * Length: triangle_count * 3
   */
  indices(): Uint32Array;
  /**
   * Returns the vertex normals as a Float32Array, if available.
   *
   * Format: [nx, ny, nz, nx, ny, nz, ...]
   * Length: vertex_count * 3
   */
  normals(): Float32Array | undefined;
  /**
   * Returns true if the mesh is empty.
   */
  is_empty(): boolean;
  /**
   * Returns the vertex positions as a Float32Array.
   *
   * Format: [x, y, z, x, y, z, ...]
   * Length: vertex_count * 3
   */
  vertices(): Float32Array;
  /**
   * Returns the number of vertices.
   */
  readonly vertex_count: number;
  /**
   * Returns the number of triangles.
   */
  readonly triangle_count: number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly render_from_cst: (a: number, b: number) => [number, number, number];
  readonly init: () => void;
  readonly __wbg_meshhandle_free: (a: number, b: number) => void;
  readonly meshhandle_colors: (a: number) => any;
  readonly meshhandle_has_colors: (a: number) => number;
  readonly meshhandle_has_normals: (a: number) => number;
  readonly meshhandle_indices: (a: number) => any;
  readonly meshhandle_is_empty: (a: number) => number;
  readonly meshhandle_normals: (a: number) => any;
  readonly meshhandle_triangle_count: (a: number) => number;
  readonly meshhandle_vertex_count: (a: number) => number;
  readonly meshhandle_vertices: (a: number) => any;
  readonly __wbg_diagnostic_free: (a: number, b: number) => void;
  readonly diagnostic_end: (a: number) => number;
  readonly diagnostic_hint: (a: number) => [number, number];
  readonly diagnostic_message: (a: number) => [number, number];
  readonly diagnostic_severity: (a: number) => [number, number];
  readonly diagnostic_start: (a: number) => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
