import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { WorkerMessage } from '$lib/worker-types';

// Mock the compileSource function
vi.mock('$lib/wasm/openscad-compiler', () => ({
	compileSource: vi.fn()
}));

import { compileSource } from '$lib/wasm/openscad-compiler';

describe('Worker', () => {
	let onmessage: (event: MessageEvent<WorkerMessage>) => Promise<void>;
	let postMessage: any;

	beforeEach(async () => {
		// Reset mocks
		vi.resetModules();
		postMessage = vi.fn();

		// Mock global self
		// We need to cast to any because we are mocking the global scope
		(global as any).self = {
			postMessage,
			onmessage: null
		};

		// Import the worker code which will attach the onmessage handler
		// We use a query parameter to ensure a fresh import if needed, 
        // though vi.resetModules() should handle it.
		await import('./index');
		onmessage = (global as any).self.onmessage;
	});

	it('handles COMPILE message successfully', async () => {
		const mockResult = { kind: 'success', vertices: new Float64Array([1, 2, 3]) };
		(compileSource as any).mockResolvedValue(mockResult);

		const message: WorkerMessage = { type: 'COMPILE', source: 'cube(10);' };
		await onmessage({ data: message } as MessageEvent);

		expect(compileSource).toHaveBeenCalledWith('cube(10);');
		expect(postMessage).toHaveBeenCalledWith(
			{ type: 'COMPILE_RESULT', result: mockResult },
			[mockResult.vertices.buffer]
		);
	});

	it('handles COMPILE message failure', async () => {
		const mockResult = { kind: 'failure', diagnostics: [] };
		(compileSource as any).mockResolvedValue(mockResult);

		const message: WorkerMessage = { type: 'COMPILE', source: 'invalid' };
		await onmessage({ data: message } as MessageEvent);

		expect(compileSource).toHaveBeenCalledWith('invalid');
		expect(postMessage).toHaveBeenCalledWith(
			{ type: 'COMPILE_RESULT', result: mockResult }
		);
	});

    it('handles exceptions during compilation', async () => {
        (compileSource as any).mockRejectedValue(new Error('Compilation crashed'));

        const message: WorkerMessage = { type: 'COMPILE', source: 'crash' };
        await onmessage({ data: message } as MessageEvent);

        expect(postMessage).toHaveBeenCalledWith(
            { type: 'ERROR', error: 'Compilation crashed' }
        );
    });
});
