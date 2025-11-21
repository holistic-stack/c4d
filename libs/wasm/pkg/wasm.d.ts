/* tslint:disable */
/* eslint-disable */
/**
 * Installs a panic hook that forwards Rust panics to the browser console.
 *
 * # Examples
 * ```no_run
 * // In JavaScript: import and call once at startup.
 * // import { init_panic_hook } from "wasm";
 * // init_panic_hook();
 * ```
 */
export function init_panic_hook(): void;
/**
 * Returns the default tessellation segment count used by the geometry
 * pipeline. This is currently a thin wrapper around a shared constant.
 *
 * # Examples
 * ```
 * let segments = wasm::default_segments();
 * assert!(segments >= 3);
 * ```
 */
export function default_segments(): number;
/**
 * Compiles OpenSCAD source and returns the number of geometry nodes
 * produced by the current evaluator pipeline.
 *
 * This function is the primary entry point used from JavaScript. For Rust
 * tests, prefer `compile_and_count_nodes_internal`, which exposes Rust
 * error types directly.
 *
 * # Errors
 * Returns a JavaScript error value containing a human-readable message
 * when evaluation fails.
 *
 * # Examples
 * ```no_run
 * // In JavaScript: await compile_and_count_nodes("cube(1);");
 * ```
 */
export function compile_and_count_nodes(source: string): number;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly default_segments: () => number;
  readonly compile_and_count_nodes: (a: number, b: number) => [number, number, number];
  readonly init_panic_hook: () => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
