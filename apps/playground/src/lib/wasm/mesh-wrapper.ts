import init, { compile_and_count_nodes, compile_and_render } from '$wasm/wasm.js';
import { wasmNotInitializedMessage } from '$lib/constants/app-config';

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
 * Compiles OpenSCAD source through the wasm evaluator and returns metadata.
 *
 * @example
 * await initWasm();
 * const result = compile('cube(1);');
 * console.log(result.nodeCount);
 */
export function compile(source: string): MeshHandle {
    if (!wasmInitialized) {
        throw new Error(wasmNotInitializedMessage);
    }

    console.log('[wasm] compile() called with source:', source);
    const nodeCount = compile_and_count_nodes(source);
    console.log('[wasm] compile() result nodeCount:', nodeCount);

    const wasmMesh = compile_and_render(source);
    const vertexCount = wasmMesh.vertex_count();
    const triangleCount = wasmMesh.triangle_count();
    const vertices = wasmMesh.vertices();
    const indices = wasmMesh.indices();

    const result: MeshHandle = { nodeCount, vertexCount, triangleCount, vertices, indices };
    console.log('[wasm] compile() mesh metrics:', result);
    return result;
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
