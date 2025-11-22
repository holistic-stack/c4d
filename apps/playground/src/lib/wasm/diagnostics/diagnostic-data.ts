
/**
 * Severity levels for diagnostics, mirroring the Rust/WASM enum.
 *
 * @example
 * const severity = Severity.Error;
 */
export enum Severity {
    Error = 0,
    Warning = 1,
    Info = 2,
    Hint = 3
}

/**
 * Plain Object representation of a diagnostic message.
 * This matches the structure emitted by the WASM pipeline.
 *
 * @example
 * const diag: DiagnosticData = {
 *   severity: Severity.Error,
 *   message: "Syntax error",
 *   start: 0,
 *   end: 5
 * };
 */
export interface DiagnosticData {
    severity: Severity;
    message: string;
    start: number;
    end: number;
    hint?: string;
}

/**
 * Represents a successful compilation result containing the mesh handle.
 */
export interface CompileSuccess<T> {
    type: 'success';
    data: T; // generic to allow passing MeshHandle
}

/**
 * Represents a failed compilation result containing diagnostics.
 */
export interface CompileError {
    type: 'error';
    diagnostics: DiagnosticData[];
}

/**
 * Discriminated union of compilation results.
 */
export type CompileResult<T> = CompileSuccess<T> | CompileError;

/**
 * Type guard to check if a result is a success.
 */
export function isCompileSuccess<T>(result: CompileResult<T>): result is CompileSuccess<T> {
    return result.type === 'success';
}

/**
 * Type guard to check if a result is an error.
 */
export function isCompileError<T>(result: CompileResult<T>): result is CompileError {
    return result.type === 'error';
}
