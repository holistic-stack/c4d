import init, { compile_and_count_nodes, compile_and_render, Severity } from '$wasm/wasm.js';
import { wasmNotInitializedMessage } from '$lib/constants/app-config';
import type { CompileResult, DiagnosticData } from './diagnostics/diagnostic-data';

/**
 * Supported parameter type for the wasm-bindgen initializer allowing dependency injection in tests.
 *
 * @example
 * const moduleBytes = new Uint8Array(await fetch('wasm_bg.wasm').then((res) => res.arrayBuffer()));
 * await initWasm(moduleBytes);
 */
export type WasmInitParameter = Parameters<typeof init>[0];

/**
 * Lightweight handle describing the node count emitted by WASM compilation.
 *
 * @example
 * const handle: MeshHandle = { nodeCount: 1 };
 */
export interface MeshHandle {
    /** Total number of nodes produced by the current evaluator pipeline. */
    nodeCount: number;
    /** Total number of vertices in the generated mesh. */
    vertexCount: number;
    /** Total number of triangles (faces) in the generated mesh. */
    triangleCount: number;
    /** Flattened vertex positions as [x0, y0, z0, x1, y1, z1, ...]. */
    vertices: Float32Array;
    /** Triangle indices referencing the vertex array. */
    indices: Uint32Array;
}

/**
 * Tracks whether the wasm-bindgen glue code has already initialized.
 *
 * @example
 * if (wasmInitialized) {
 *     console.log('ready');
 * }
 */
let wasmInitialized = false;

/**
 * Ensures the wasm bundle is initialized exactly once before usage.
 *
 * @example
 * await initWasm();
 */
export async function initWasm(moduleOrPath?: WasmInitParameter): Promise<void> {
    if (wasmInitialized) {
        console.log('[wasm] initWasm() called but already initialized');
        return;
    }

    console.log('[wasm] initWasm() start', { moduleOrPath });
    await init(moduleOrPath);
    console.log('[wasm] initWasm() complete');
    wasmInitialized = true;
}

/**
 * Compiles OpenSCAD source through the wasm evaluator and returns a discriminated union result.
 *
 * @example
 * await initWasm();
 * const result = compile('cube(1);');
 * if (result.type === 'success') {
 *   console.log(result.data.nodeCount);
 * } else {
 *   console.error(result.diagnostics);
 * }
 */
export function compile(source: string): CompileResult<MeshHandle> {
    if (!wasmInitialized) {
        throw new Error(wasmNotInitializedMessage);
    }

    console.log('[wasm] compile() called with source:', source);

    let nodeCount = 0;
    try {
        nodeCount = compile_and_count_nodes(source);
    } catch (e) {
        console.warn('[wasm] compile_and_count_nodes failed, will be caught by compile_and_render if needed', e);
    }

    console.log('[wasm] compile() result nodeCount:', nodeCount);

    try {
        const wasmMesh = compile_and_render(source);
        const vertexCount = wasmMesh.vertex_count();
        const triangleCount = wasmMesh.triangle_count();
        const vertices = wasmMesh.vertices();
        const indices = wasmMesh.indices();

        const handle: MeshHandle = { nodeCount, vertexCount, triangleCount, vertices, indices };
        console.log('[wasm] compile() mesh metrics:', handle);

        return {
            type: 'success',
            data: handle
        };

    } catch (error: unknown) {
        console.error('[wasm] compile_and_render failed with:', error);

        // Assert it matches { diagnostics }
        const payload = error as { diagnostics?: DiagnosticData[] };
        if (payload && Array.isArray(payload.diagnostics)) {
            return {
                type: 'error',
                diagnostics: payload.diagnostics
            };
        }

        // Rethrow if it's not a structured diagnostic error (e.g. panic or other js error)
        throw error;
    }
}

/**
 * Resets the local initialization flag to support deterministic tests.
 *
 * @example
 * resetWasmState();
 */
export function resetWasmState(): void {
    wasmInitialized = false;
}

export { Severity };
