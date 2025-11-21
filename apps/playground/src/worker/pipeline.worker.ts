import { initWasm, compile } from '../lib/wasm/mesh-wrapper';

self.onmessage = async (event: MessageEvent) => {
    const { type, payload } = event.data;

    try {
        if (type === 'init') {
            await initWasm();
            self.postMessage({ type: 'init_complete' });
        } else if (type === 'compile') {
            const result = compile(payload);
            self.postMessage({ type: 'compile_success', payload: result });
        }
    } catch (error) {
        self.postMessage({
            type: 'error',
            payload: error instanceof Error ? error.message : String(error)
        });
    }
};
