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
}

/**
 * Mesh data containing vertices and indices for rendering.
 */
export interface RenderableMesh {
    vertices: Float32Array;
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
        return;
    }

    await init(moduleOrPath);
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

    const count = compile_and_count_nodes(source);
    return { nodeCount: count };
}

/**
 * Compiles OpenSCAD source through the wasm evaluator and returns mesh data.
 *
 * @example
 * await initWasm();
 * const mesh = compileMesh('cube(1);');
 * console.log(mesh.vertices.length);
 */
export function compileMesh(source: string): RenderableMesh {
    if (!wasmInitialized) {
        throw new Error(wasmNotInitializedMessage);
    }

    const handle = compile_and_render(source);

    // Extract data before freeing the handle
    const vertices = handle.vertices();
    const indices = handle.indices();

    handle.free();

    return {
        vertices: new Float32Array(vertices),
        indices: new Uint32Array(indices)
    };
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
