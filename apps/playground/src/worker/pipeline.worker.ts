import { initWasm, compileMesh } from '../lib/wasm/mesh-wrapper';

self.onmessage = async (event: MessageEvent) => {
    const { type, payload } = event.data;

    try {
        if (type === 'init') {
            await initWasm();
            self.postMessage({ type: 'init_complete' });
        } else if (type === 'compile') {
            // Formerly used `compile` which only returned node count.
            // Now using `compileMesh` to get geometry data.
            const result = compileMesh(payload);

            // Transfer buffers to avoid copying if possible, though here we just send the object.
            // To optimize, we could use transferables: [result.vertices.buffer, result.indices.buffer]
            self.postMessage(
                { type: 'compile_success', payload: result },
                { transfer: [result.vertices.buffer, result.indices.buffer] }
            );
        }
    } catch (error) {
        self.postMessage({
            type: 'error',
            payload: error instanceof Error ? error.message : String(error)
        });
    }
};
