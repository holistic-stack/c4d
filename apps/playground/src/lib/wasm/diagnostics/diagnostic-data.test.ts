
import { describe, it, expect } from 'vitest';
import {
    Severity,
    isCompileSuccess,
    isCompileError,
    type CompileResult,
    type DiagnosticData
} from './diagnostic-data';

/**
 * Tests for diagnostic data structures and type guards.
 */
describe('Diagnostic Types', () => {
    it('should identify success results correctly', () => {
        const success: CompileResult<string> = {
            type: 'success',
            data: 'mesh-data'
        };

        expect(isCompileSuccess(success)).toBe(true);
        expect(isCompileError(success)).toBe(false);
    });

    it('should identify error results correctly', () => {
        const diag: DiagnosticData = {
            severity: Severity.Error,
            message: 'test error',
            start: 0,
            end: 10
        };

        const error: CompileResult<string> = {
            type: 'error',
            diagnostics: [diag]
        };

        expect(isCompileSuccess(error)).toBe(false);
        expect(isCompileError(error)).toBe(true);
    });

    it('should handle empty diagnostics list in error', () => {
        const error: CompileResult<string> = {
            type: 'error',
            diagnostics: []
        };
        expect(isCompileError(error)).toBe(true);
    });

    it('should have correct severity enum values', () => {
        // These must match the Rust side logic implicitly
        expect(Severity.Error).toBe(0);
        expect(Severity.Warning).toBe(1);
    });
});
