import { compileSource } from '$lib/wasm/openscad-compiler';
import type { WorkerMessage, WorkerResponse } from '$lib/worker-types';

/**
 * Web Worker entry point for the OpenSCAD pipeline.
 * Handles compilation requests and returns results (mesh or diagnostics).
 */

self.onmessage = async (event: MessageEvent<WorkerMessage>) => {
	const { data } = event;

	switch (data.type) {
		case 'COMPILE':
			await handleCompile(data.source);
			break;
		case 'CANCEL':
			// TODO: Implement cancellation logic if supported by WASM
			break;
	}
};

async function handleCompile(source: string): Promise<void> {
	try {
		const result = await compileSource(source);

		if (result.kind === 'success') {
			// Transfer the vertex buffer to avoid copying
			const response: WorkerResponse = { type: 'COMPILE_RESULT', result };
			self.postMessage(response, [result.vertices.buffer]);
		} else {
			const response: WorkerResponse = { type: 'COMPILE_RESULT', result };
			self.postMessage(response);
		}
	} catch (err) {
		const error = err instanceof Error ? err.message : String(err);
		const response: WorkerResponse = { type: 'ERROR', error };
		self.postMessage(response);
	}
}
