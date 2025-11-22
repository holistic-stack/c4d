
import type { DiagnosticData } from '../../lib/wasm/diagnostics/diagnostic-data';
import type { MeshHandle } from '../../lib/wasm/mesh-wrapper';

/**
 * Message sent from the worker to the main thread when initialization starts.
 */
export interface InitMessage {
    type: 'init';
}

/**
 * Message sent from the worker to the main thread when initialization completes.
 */
export interface InitCompleteMessage {
    type: 'init_complete';
}

/**
 * Message sent from the main thread to the worker to request compilation.
 */
export interface CompileRequestMessage {
    type: 'compile';
    payload: string; // Source code
}

/**
 * Message sent from the worker to the main thread on successful compilation.
 */
export interface CompileSuccessMessage {
    type: 'compile_success';
    payload: MeshHandle;
}

/**
 * Message sent from the worker to the main thread on compilation error.
 */
export interface CompileErrorMessage {
    type: 'compile_error';
    payload: DiagnosticData[];
}

/**
 * Generic error message for unexpected failures.
 */
export interface ErrorMessage {
    type: 'error';
    payload: string;
}

/**
 * Union of all messages sent TO the worker.
 */
export type WorkerRequest = InitMessage | CompileRequestMessage;

/**
 * Union of all messages sent FROM the worker.
 */
export type WorkerResponse =
    | InitCompleteMessage
    | CompileSuccessMessage
    | CompileErrorMessage
    | ErrorMessage;
