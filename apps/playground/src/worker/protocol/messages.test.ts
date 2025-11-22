
import { describe, it, expect } from 'vitest';
import type { WorkerRequest, WorkerResponse } from './messages';
import { Severity } from '../../lib/wasm/diagnostics/diagnostic-data';
import type { MeshHandle } from '../../lib/wasm/mesh-wrapper';

/**
 * Tests for worker protocol messages to ensure type consistency.
 */
describe('Worker Protocol', () => {
    it('should structure compile request correctly', () => {
        const req: WorkerRequest = {
            type: 'compile',
            payload: 'cube(10);'
        };
        expect(req.type).toBe('compile');
        expect(req.payload).toBe('cube(10);');
    });

    it('should structure compile error response correctly', () => {
        const res: WorkerResponse = {
            type: 'compile_error',
            payload: [{
                severity: Severity.Error,
                message: 'failed',
                start: 0,
                end: 1
            }]
        };
        expect(res.type).toBe('compile_error');
        expect(res.payload[0].severity).toBe(Severity.Error);
    });

    it('should structure compile success response correctly', () => {
        const mockHandle: MeshHandle = {
            nodeCount: 1,
            vertexCount: 3,
            triangleCount: 1,
            vertices: new Float32Array([0,0,0]),
            indices: new Uint32Array([0,1,2])
        };

        const res: WorkerResponse = {
            type: 'compile_success',
            payload: mockHandle
        };
        expect(res.type).toBe('compile_success');
        expect(res.payload.nodeCount).toBe(1);
    });
});
