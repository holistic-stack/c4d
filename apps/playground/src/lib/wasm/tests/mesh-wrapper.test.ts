
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { compile, initWasm, resetWasmState, type MeshHandle } from '../mesh-wrapper';
import { Severity, type DiagnosticData, type CompileResult } from '../diagnostics/diagnostic-data';
import * as wasmModule from '$wasm/wasm.js';

// Mock the entire WASM module
vi.mock('$wasm/wasm.js', () => {
    return {
        default: vi.fn().mockResolvedValue({}),
        compile_and_count_nodes: vi.fn(),
        compile_and_render: vi.fn(),
        Severity: {
            Error: 0,
            Warning: 1,
            Info: 2,
            Hint: 3
        }
    };
});

describe('mesh-wrapper', () => {
    beforeEach(() => {
        resetWasmState();
        vi.clearAllMocks();
    });

    it('should initialize wasm only once', async () => {
        const initSpy = vi.mocked(wasmModule.default);
        await initWasm();
        await initWasm();
        expect(initSpy).toHaveBeenCalledTimes(1);
    });

    it('should return success result on valid compilation', async () => {
        await initWasm();

        const mockMesh = {
            vertex_count: () => 3,
            triangle_count: () => 1,
            vertices: () => new Float32Array([0,0,0]),
            indices: () => new Uint32Array([0,1,2]),
            free: () => {}
        };

        vi.mocked(wasmModule.compile_and_count_nodes).mockReturnValue(1);
        vi.mocked(wasmModule.compile_and_render).mockReturnValue(mockMesh as any);

        const result = compile('cube(10);');

        expect(result.type).toBe('success');
        if (result.type === 'success') {
            expect(result.data.nodeCount).toBe(1);
            expect(result.data.vertexCount).toBe(3);
        }
    });

    it('should return error result with diagnostics when compilation fails', async () => {
        await initWasm();

        const diag: DiagnosticData = {
            severity: Severity.Error,
            message: 'Syntax error',
            start: 0,
            end: 5
        };

        const errorPayload = { diagnostics: [diag] };

        // Mock compile_and_count_nodes to throw (or succeed, doesn't matter much as we focus on render failure)
        vi.mocked(wasmModule.compile_and_count_nodes).mockReturnValue(0);

        // Mock compile_and_render to throw the structured error
        vi.mocked(wasmModule.compile_and_render).mockImplementation(() => {
            throw errorPayload;
        });

        const result = compile('invalid');

        expect(result.type).toBe('error');
        if (result.type === 'error') {
            expect(result.diagnostics).toHaveLength(1);
            expect(result.diagnostics[0].message).toBe('Syntax error');
        }
    });

    it('should handle unstructured errors gracefully if possible or rethrow if truly unknown', async () => {
        await initWasm();

        const genericError = new Error('Random failure');
        vi.mocked(wasmModule.compile_and_render).mockImplementation(() => {
            throw genericError;
        });

        expect(() => compile('crash')).toThrow('Random failure');
    });
});
