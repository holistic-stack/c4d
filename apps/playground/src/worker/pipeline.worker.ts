import { initWasm, compile, type DiagnosticData } from '../lib/wasm/mesh-wrapper';

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
    } catch (error: unknown) {
        console.error('[worker] error during message handling', error);

        // Check if it's a structured diagnostic error
        // The error thrown by compile() contains diagnostics as POJOs (DiagnosticData)
        const errorObj = error as { diagnostics?: DiagnosticData[] };
        if (errorObj && Array.isArray(errorObj.diagnostics)) {
            self.postMessage({
                type: 'compile_error',
                payload: errorObj.diagnostics
            });
        } else {
            // Fallback for other errors
            self.postMessage({
                type: 'error',
                payload: error instanceof Error ? error.message : String(error)
            });
        }
    }
};
