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
 * Compiles OpenSCAD source and renders it to a mesh.
 *
 * This is the main entry point for the pipeline. It parses the source,
 * evaluates it, and generates a mesh suitable for GPU rendering.
 *
 * # Errors
 * Returns a JavaScript error containing diagnostics if compilation fails.
 *
 * # Examples
 * ```no_run
 * // In JavaScript:
 * // try {
 * //   const mesh = await compile_and_render("cube([2, 2, 2]);");
 * //   console.log("Vertices:", mesh.vertex_count());
 * // } catch (error) {
 * //   console.error("Compilation failed:", error);
 * // }
 * ```
 */
export function compile_and_render(source: string): MeshHandle;
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
/**
 * Diagnostic severity for JavaScript.
 */
export enum Severity {
  Error = 0,
  Warning = 1,
  Info = 2,
}
/**
 * A diagnostic message for JavaScript.
 *
 * # Examples
 * ```no_run
 * // In JavaScript:
 * // const diag = result.diagnostics[0];
 * // console.log(diag.message());
 * // console.log(diag.start(), diag.end());
 * ```
 */
export class Diagnostic {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Returns the end position in the source.
   */
  end(): number;
  /**
   * Returns the hint, if any.
   */
  hint(): string | undefined;
  /**
   * Returns the start position in the source.
   */
  start(): number;
  /**
   * Returns the diagnostic message.
   */
  message(): string;
  /**
   * Returns the severity of the diagnostic.
   */
  severity(): Severity;
}
/**
 * A collection of diagnostics.
 */
export class DiagnosticList {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Returns a diagnostic by index.
   */
  get(index: number): Diagnostic | undefined;
  /**
   * Returns the number of diagnostics.
   */
  len(): number;
  /**
   * Returns true if there are no diagnostics.
   */
  is_empty(): boolean;
}
/**
 * Mesh handle returned from compilation.
 *
 * Contains vertex and index counts for the rendered mesh.
 *
 * # Examples
 * ```no_run
 * // In JavaScript:
 * // const result = await compile_and_render("cube([1, 1, 1]);");
 * // console.log(result.vertex_count(), result.triangle_count());
 * ```
 */
export class MeshHandle {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Returns the number of vertices in the mesh.
   */
  vertex_count(): number;
  /**
   * Returns the number of triangles in the mesh.
   */
  triangle_count(): number;
  /**
   * Returns the index buffer as a Uint32Array.
   */
  indices(): Uint32Array;
  /**
   * Returns the vertex buffer as a Float32Array.
   */
  vertices(): Float32Array;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_meshhandle_free: (a: number, b: number) => void;
  readonly compile_and_count_nodes: (a: number, b: number) => [number, number, number];
  readonly compile_and_render: (a: number, b: number) => [number, number, number];
  readonly default_segments: () => number;
  readonly meshhandle_indices: (a: number) => [number, number];
  readonly meshhandle_triangle_count: (a: number) => number;
  readonly meshhandle_vertex_count: (a: number) => number;
  readonly meshhandle_vertices: (a: number) => [number, number];
  readonly init_panic_hook: () => void;
  readonly abort: () => void;
  readonly calloc: (a: number, b: number) => number;
  readonly clock: () => number;
  readonly close: (a: number) => number;
  readonly dup: (a: number) => number;
  readonly fclose: (a: number) => number;
  readonly fdopen: (a: number, b: number) => number;
  readonly fputc: (a: number, b: number) => number;
  readonly fputs: (a: number, b: number) => number;
  readonly free: (a: number) => void;
  readonly fwrite: (a: number, b: number, c: number, d: number) => number;
  readonly isalnum: (a: number) => number;
  readonly isalpha: (a: number) => number;
  readonly isdigit: (a: number) => number;
  readonly isprint: (a: number) => number;
  readonly isspace: (a: number) => number;
  readonly malloc: (a: number) => number;
  readonly memchr: (a: number, b: number, c: number) => number;
  readonly memcmp: (a: number, b: number, c: number) => number;
  readonly memcpy: (a: number, b: number, c: number) => number;
  readonly memmove: (a: number, b: number, c: number) => number;
  readonly memset: (a: number, b: number, c: number) => number;
  readonly read: (a: number, b: number, c: number) => number;
  readonly realloc: (a: number, b: number) => number;
  readonly strcat: (a: number, b: number) => number;
  readonly strlen: (a: number) => number;
  readonly strcmp: (a: number, b: number) => number;
  readonly strcpy: (a: number, b: number) => number;
  readonly strdup: (a: number) => number;
  readonly strncmp: (a: number, b: number, c: number) => number;
  readonly strncpy: (a: number, b: number, c: number) => number;
  readonly strrchr: (a: number, b: number) => number;
  readonly towlower: (a: number) => number;
  readonly towupper: (a: number) => number;
  readonly time: (a: number) => number;
  readonly iswalpha: (a: number) => number;
  readonly iswalnum: (a: number) => number;
  readonly iswdigit: (a: number) => number;
  readonly iswspace: (a: number) => number;
  readonly write: (a: number, b: number, c: number) => number;
  readonly __wbg_diagnostic_free: (a: number, b: number) => void;
  readonly __wbg_diagnosticlist_free: (a: number, b: number) => void;
  readonly diagnostic_end: (a: number) => number;
  readonly diagnostic_hint: (a: number) => [number, number];
  readonly diagnostic_message: (a: number) => [number, number];
  readonly diagnostic_severity: (a: number) => number;
  readonly diagnostic_start: (a: number) => number;
  readonly diagnosticlist_get: (a: number, b: number) => number;
  readonly diagnosticlist_is_empty: (a: number) => number;
  readonly diagnosticlist_len: (a: number) => number;
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
