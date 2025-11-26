/**
 * # Worker Module
 *
 * Exports the WorkerClient for non-blocking OpenSCAD compilation.
 *
 * ## Usage
 *
 * ```typescript
 * import { WorkerClient } from '$lib/worker';
 *
 * const client = new WorkerClient();
 * await client.initialize();
 * const result = await client.compile('cube(10);');
 * ```
 */

export { WorkerClient, type Diagnostic, type CompileResult, type CompileError } from './worker-client';
