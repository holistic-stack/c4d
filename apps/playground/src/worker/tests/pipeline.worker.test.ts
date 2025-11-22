
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import type { WorkerRequest, WorkerResponse, InitMessage, CompileRequestMessage } from '../protocol/messages';
import { Severity, type CompileResult, type DiagnosticData } from '../../lib/wasm/diagnostics/diagnostic-data';

// We need to load the worker script in a way that we can test it.
// Since it's a worker, it relies on 'self'. We can mock 'self'.

describe('Pipeline Worker', () => {
    let workerHandler: (event: MessageEvent<WorkerRequest>) => Promise<void>;
    let postedMessages: WorkerResponse[];

    // Mocks for dependencies
    const mockInitWasm = vi.fn();
    const mockCompile = vi.fn();

    beforeEach(async () => {
        postedMessages = [];

        // Mock global self
        const selfMock = {
            postMessage: (msg: WorkerResponse) => postedMessages.push(msg),
            onmessage: null as any
        };

        // Setup module mocks
        vi.doMock('../../lib/wasm/mesh-wrapper', () => ({
            initWasm: mockInitWasm,
            compile: mockCompile
        }));

        // We need to reload the worker file to pick up the mocks and new 'self' context
        // This is tricky with ESM. A simpler way is to import the module and check side effects
        // but 'pipeline.worker.ts' assigns to self.onmessage.

        // We will mock global scope for the test
        global.self = selfMock as any;

        // Load the worker code
        await import('../pipeline.worker.ts?update=' + Date.now());

        workerHandler = global.self.onmessage as any; // Cast to ensure it's not null in types
    });

    afterEach(() => {
        vi.resetModules();
        vi.clearAllMocks();
    });

    it('should handle init message', async () => {
        mockInitWasm.mockResolvedValue(undefined);

        const msg: InitMessage = { type: 'init' };
        const event = new MessageEvent('message', {
            data: msg
        });

        await workerHandler(event);

        expect(mockInitWasm).toHaveBeenCalled();
        expect(postedMessages).toHaveLength(1);
        expect(postedMessages[0].type).toBe('init_complete');
    });

    it('should handle compile success', async () => {
        const mockMesh = { nodeCount: 1 };
        mockCompile.mockReturnValue({
            type: 'success',
            data: mockMesh
        });

        const msg: CompileRequestMessage = { type: 'compile', payload: 'cube(10);' };
        const event = new MessageEvent('message', {
            data: msg
        });

        await workerHandler(event);

        expect(mockCompile).toHaveBeenCalledWith('cube(10);');
        expect(postedMessages).toHaveLength(1);
        expect(postedMessages[0].type).toBe('compile_success');
        if (postedMessages[0].type === 'compile_success') {
            expect(postedMessages[0].payload).toEqual(mockMesh);
        }
    });

    it('should handle compile error with diagnostics', async () => {
        const diags: DiagnosticData[] = [{
            severity: Severity.Error,
            message: 'Fail',
            start: 0,
            end: 1
        }];

        mockCompile.mockReturnValue({
            type: 'error',
            diagnostics: diags
        });

        const msg: CompileRequestMessage = { type: 'compile', payload: 'bad' };
        const event = new MessageEvent('message', {
            data: msg
        });

        await workerHandler(event);

        expect(postedMessages).toHaveLength(1);
        expect(postedMessages[0].type).toBe('compile_error');
        if (postedMessages[0].type === 'compile_error') {
            expect(postedMessages[0].payload).toEqual(diags);
        }
    });

    it('should catch unexpected errors', async () => {
        mockCompile.mockImplementation(() => {
            throw new Error('Crash');
        });

        const msg: CompileRequestMessage = { type: 'compile', payload: 'crash' };
        const event = new MessageEvent('message', {
            data: msg
        });

        await workerHandler(event);

        expect(postedMessages).toHaveLength(1);
        expect(postedMessages[0].type).toBe('error');
        if (postedMessages[0].type === 'error') {
            expect(postedMessages[0].payload).toBe('Crash');
        }
    });
});
