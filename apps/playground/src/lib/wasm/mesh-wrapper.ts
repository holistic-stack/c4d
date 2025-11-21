import init, { compile_and_count_nodes } from '$wasm/wasm.js';

export interface MeshHandle {
    // Placeholder for now, as we only return node count
    nodeCount: number;
}

let wasmInitialized = false;

export async function initWasm() {
    if (wasmInitialized) return;

    // We explicitly pass the URL to the wasm file to ensure Vite handles it correctly.
    // However, since we are importing from outside src, we might need to rely on 
    // wasm-bindgen's default behavior or pass a specific URL.
    // For now, let's try default init(). If it fails to find the .wasm, 
    // we might need to use `?url` import or copy files to static.
    await init();
    wasmInitialized = true;
}

export function compile(source: string): MeshHandle {
    if (!wasmInitialized) {
        throw new Error("WASM not initialized. Call initWasm() first.");
    }

    try {
        const count = compile_and_count_nodes(source);
        return { nodeCount: count };
    } catch (e) {
        console.error("WASM compilation error:", e);
        throw e;
    }
}
