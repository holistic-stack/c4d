
import { describe, it, expect } from 'vitest';
// import { render } from '@testing-library/svelte'; // Removed since not installed/configured
import { Severity } from '../../wasm/diagnostics/diagnostic-data';
import { getSeverityClass } from './utils';

describe('DiagnosticsPanel Logic', () => {
    it('should map Error severity correctly', () => {
        expect(getSeverityClass(Severity.Error)).toBe('error');
    });

    it('should map Warning severity correctly', () => {
        expect(getSeverityClass(Severity.Warning)).toBe('warning');
    });

    it('should map Info severity correctly', () => {
        expect(getSeverityClass(Severity.Info)).toBe('info');
    });

    it('should map Hint severity correctly', () => {
        expect(getSeverityClass(Severity.Hint)).toBe('hint');
    });

    it('should default to info for unknown severity', () => {
        expect(getSeverityClass(999)).toBe('info');
    });
});
