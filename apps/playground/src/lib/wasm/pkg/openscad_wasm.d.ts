/* tslint:disable */
/* eslint-disable */
/**
 * Get the WASM module version.
 *
 * ## Returns
 *
 * Version string (e.g., "0.1.0")
 *
 * ## Example (JavaScript)
 *
 * ```javascript
 * const version = get_version();
 * console.log(`WASM version: ${version}`);
 * ```
 */
export function get_version(): string;
/**
 * Render from CST JSON (main entry point).
 *
 * Accepts CST JSON from web-tree-sitter and returns mesh data.
 *
 * ## Parameters
 *
 * - `cst_json`: JSON string of CST from tree-sitter
 *
 * ## Returns
 *
 * JavaScript object with typed arrays:
 * - `success`: boolean
 * - `vertices`: Float32Array (x, y, z positions)
 * - `indices`: Uint32Array (triangle indices)
 * - `normals`: Float32Array (x, y, z normals)
 * - `vertexCount`: number
 * - `triangleCount`: number
 * - `renderTimeMs`: number
 *
 * ## Example (JavaScript)
 *
 * ```javascript
 * const cstJson = parseToJson('cube(20);');
 * const result = render_from_cst(cstJson);
 * if (result.success) {
 *     scene.updateMesh(result.vertices, result.indices, result.normals);
 * }
 * ```
 */
export function render_from_cst(cst_json: string): any;
/**
 * Initialize the WASM module.
 *
 * Sets up panic hook for better error messages in browser console.
 * Call this once before using any other functions.
 *
 * ## Example (JavaScript)
 *
 * ```javascript
 * import init from './openscad_wasm.js';
 * await init();
 * ```
 */
export function wasm_init(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly get_version: () => [number, number];
  readonly render_from_cst: (a: number, b: number) => any;
  readonly wasm_init: () => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
