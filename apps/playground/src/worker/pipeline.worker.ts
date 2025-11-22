import { initWasm, compile } from '../lib/wasm/mesh-wrapper';

console.log('[worker] pipeline.worker loaded');

self.onmessage = async (event: MessageEvent) => {
    const { type, payload } = event.data;
    console.log('[worker] received message', { type, payload });

    try {
        if (type === 'init') {
            console.log('[worker] initWasm() start');
            await initWasm();
            console.log('[worker] initWasm() complete');
            self.postMessage({ type: 'init_complete' });
        } else if (type === 'compile') {
            console.log('[worker] compile() called with source:', payload);
            const result = compile(payload);
            console.log('[worker] compile() result:', result);
            self.postMessage({ type: 'compile_success', payload: result });
        }
    } catch (error) {
        console.error('[worker] error during message handling', error);
        self.postMessage({
            type: 'error',
            payload: error instanceof Error ? error.message : String(error)
        });
    }
};
