import { beforeEach, afterEach, describe, expect, it } from 'vitest';

import { setupWasmFetchInterceptor, type WasmFetchInterceptor } from '$lib/test-utils/wasm-fetch';
import { compileSource, type CompileResult } from './index';

/**
 * Integration tests for the high-level OpenSCAD compiler wrapper.
 *
 * These tests exercise the real Rust/WASM pipeline end-to-end:
 * - WebAssembly is loaded from the local `openscad_wasm_bg.wasm` file.
 * - `compileSource` calls into the `compile_and_render` export.
 * - Successful and failing compilations are validated without mocks.
 *
 * @example
 * ```ts
 * const result = await compileSource('cube(10);');
 * if (result.kind === 'success') {
 *   console.log(result.vertices.length);
 * }
 * ```
 */
let interceptor: WasmFetchInterceptor;

beforeEach(() => {
  interceptor = setupWasmFetchInterceptor({
    map: { 'openscad_wasm_bg.wasm': 'src/lib/wasm/pkg/openscad_wasm_bg.wasm' },
    allowPassThrough: true
  });
});

afterEach(() => {
  interceptor.restore();
});

/**
 * Helper to assert that a compile result is a successful mesh.
 *
 * @example
 * ```ts
 * const result = await compileSource('cube(10);');
 * assertSuccess(result);
 * ```
 */
function assertSuccess(result: CompileResult): void {
  if (result.kind === 'failure') {
    console.error('Compilation failed:', JSON.stringify(result.diagnostics, null, 2));
  }
  expect(result.kind).toBe('success');

  if (result.kind === 'success') {
    expect(result.vertices.length).toBeGreaterThan(0);
    expect(result.vertices.length % 9).toBe(0);
  }
}

/**
 * Helper to assert that a compile result contains diagnostics.
 *
 * @example
 * ```ts
 * const result = await compileSource('cube(');
 * assertFailure(result);
 * ```
 */
function assertFailure(result: CompileResult): void {
  expect(result.kind).toBe('failure');

  if (result.kind === 'failure') {
    expect(result.diagnostics.length).toBeGreaterThan(0);
    expect(result.diagnostics[0]?.message.length).toBeGreaterThan(0);
  }
}

describe('compileSource success', () => {
  it('compiles a simple cube source through the pipeline', async () => {
    const result = await compileSource('cube(10);');
    assertSuccess(result);
  });
});

describe('compileSource failure', () => {
  it('returns diagnostics for clearly invalid OpenSCAD code', async () => {
    const result = await compileSource('cube(');
    assertFailure(result);
  });
});
