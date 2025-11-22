import init, { compile_and_count_nodes } from '$wasm/wasm.js';
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
 * Resets the local initialization flag to support deterministic tests.
 *
 * @example
 * resetWasmState();
 */
export function resetWasmState(): void {
    wasmInitialized = false;
}
