/**
 * Error message shared across the app whenever WASM helpers are invoked before initialization.
 *
 * @example
 * import { wasmNotInitializedMessage } from '$lib/constants/app-config';
 * throw new Error(wasmNotInitializedMessage);
 */
export const wasmNotInitializedMessage = 'WASM not initialized. Call initWasm() first.';
