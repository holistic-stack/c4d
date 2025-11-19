import type { CompileResult } from './wasm/openscad-compiler';

/**
 * Message sent from the main thread to the worker.
 */
export type WorkerMessage =
  | { type: 'COMPILE'; source: string }
  | { type: 'CANCEL' };

/**
 * Message sent from the worker back to the main thread.
 */
export type WorkerResponse =
  | { type: 'COMPILE_RESULT'; result: CompileResult }
  | { type: 'ERROR'; error: string };
