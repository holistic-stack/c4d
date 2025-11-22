import { initWasm, compile } from '../lib/wasm/mesh-wrapper';
import type {
    WorkerRequest,
    WorkerResponse,
    CompileSuccessMessage,
    CompileErrorMessage,
    ErrorMessage,
    InitCompleteMessage
} from './protocol/messages';

console.log('[worker] pipeline.worker loaded');

// Helper to post typed messages
function post(message: WorkerResponse) {
    self.postMessage(message);
}

self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
    const { type } = event.data;
    console.log('[worker] received message', event.data);

    try {
        if (type === 'init') {
            console.log('[worker] initWasm() start');
            await initWasm();
            console.log('[worker] initWasm() complete');

            const msg: InitCompleteMessage = { type: 'init_complete' };
            post(msg);
        }
        else if (type === 'compile') {
            const { payload } = event.data;
            console.log('[worker] compile() called with source:', payload);

            const result = compile(payload);

            if (result.type === 'success') {
                console.log('[worker] compile() success:', result.data);
                const msg: CompileSuccessMessage = {
                    type: 'compile_success',
                    payload: result.data
                };
                post(msg);
            } else {
                console.log('[worker] compile() error:', result.diagnostics);
                const msg: CompileErrorMessage = {
                    type: 'compile_error',
                    payload: result.diagnostics
                };
                post(msg);
            }
        }
    } catch (error: unknown) {
        console.error('[worker] error during message handling', error);

        const msg: ErrorMessage = {
            type: 'error',
            payload: error instanceof Error ? error.message : String(error)
        };
        post(msg);
    }
};
