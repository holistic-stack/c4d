
import { Severity } from '../../wasm/diagnostics/diagnostic-data';

export function getSeverityClass(severity: number): string {
    switch (severity) {
        case Severity.Error: return 'error';
        case Severity.Warning: return 'warning';
        case Severity.Info: return 'info';
        case Severity.Hint: return 'hint';
        default: return 'info';
    }
}
