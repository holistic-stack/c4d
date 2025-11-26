/**
 * # WASM Types
 *
 * TypeScript type definitions for the OpenSCAD WASM module.
 * These types mirror the Rust structures exposed via wasm-bindgen.
 */

/**
 * Mesh handle returned from the WASM module.
 * Provides access to vertex and index buffers for Three.js rendering.
 */
export interface MeshHandle {
	/** Number of vertices in the mesh */
	readonly vertex_count: number;
	/** Number of triangles in the mesh */
	readonly triangle_count: number;
	/** Returns vertex positions as Float32Array [x, y, z, ...] */
	vertices(): Float32Array;
	/** Returns triangle indices as Uint32Array [i0, i1, i2, ...] */
	indices(): Uint32Array;
	/** Returns vertex normals if available */
	normals(): Float32Array | undefined;
	/** Returns vertex colors if available [r, g, b, a, ...] */
	colors(): Float32Array | undefined;
	/** Returns true if the mesh has normals */
	has_normals(): boolean;
	/** Returns true if the mesh has colors */
	has_colors(): boolean;
	/** Returns true if the mesh is empty */
	is_empty(): boolean;
}

/**
 * Diagnostic severity levels.
 */
export type DiagnosticSeverity = 'error' | 'warning' | 'info' | 'hint';

/**
 * Diagnostic information from the compiler.
 * Properties are readonly getters in the WASM class.
 */
export interface Diagnostic {
	/** Severity level */
	readonly severity: string;
	/** Error message */
	readonly message: string;
	/** Start byte offset in source */
	readonly start: number;
	/** End byte offset in source */
	readonly end: number;
	/** Optional hint for fixing the issue */
	readonly hint: string | undefined;
}

/**
 * Error payload thrown by compile_and_render on failure.
 */
export interface CompileError {
	/** Array of diagnostics describing the errors */
	diagnostics: Diagnostic[];
}

/**
 * The WASM module interface.
 * 
 * Uses the browser-safe CST-based API that accepts pre-parsed
 * syntax trees from web-tree-sitter.
 */
export interface OpenSCADWasm {
	/** 
	 * Renders a mesh from a serialized CST (browser-safe).
	 * 
	 * @param cst_json - JSON string containing the serialized CST from web-tree-sitter
	 * @returns MeshHandle containing vertex and index buffers
	 * @throws CompileError if parsing, evaluation, or mesh generation fails
	 */
	render_from_cst(cst_json: string): MeshHandle;
	/** Initialize the WASM module */
	init(): void;
}

/**
 * Type guard to check if an error is a CompileError.
 *
 * @param error - The error to check
 * @returns True if the error has a diagnostics array
 *
 * @example
 * ```typescript
 * try {
 *   const mesh = wasm.compile_and_render(source);
 * } catch (error) {
 *   if (isCompileError(error)) {
 *     for (const diag of error.diagnostics) {
 *       console.error(diag.message);
 *     }
 *   }
 * }
 * ```
 */
export function isCompileError(error: unknown): error is CompileError {
	return (
		typeof error === 'object' &&
		error !== null &&
		'diagnostics' in error &&
		Array.isArray((error as CompileError).diagnostics)
	);
}
