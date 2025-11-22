/**
 * Contract tests for the wasm mesh wrapper ensuring proper initialization semantics.
 *
 * @example
 * // Run with Vitest to verify behavior
 * // pnpm vitest run src/lib/wasm/mesh-wrapper.test.ts
 */
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { afterEach, describe, expect, it } from 'vitest';
import { compile, initWasm, resetWasmState } from './mesh-wrapper';

/** Absolute path to the compiled wasm binary shared across tests. */
const wasmBinaryPath = fileURLToPath(
  new URL('../../../../../libs/wasm/pkg/wasm_bg.wasm', import.meta.url)
);

/**
 * Lazily loads wasm bytes from disk to exercise the real initialization pathway.
 *
 * @example
 * const bytes = await loadWasmBytes();
 */
async function loadWasmBytes(): Promise<Uint8Array> {
  const buffer = await readFile(wasmBinaryPath);
  return new Uint8Array(buffer);
}

/**
 * Ensures shared state is reset between specs for deterministic execution.
 *
 * @example
 * afterEach(resetWasmState);
 */
afterEach(() => {
  resetWasmState();
});

/**
 * Groups initialization-specific edge cases validated with real wasm binaries.
 *
 * @example
 * describe('initWasm', () => {
 *   // setup verifications
 * });
 */
describe('initWasm', () => {
  /**
   * Validates that custom wasm bytes can be provided explicitly.
   *
   * @example
   * await initWasm(customBytes);
   */
  it('initializes using manually provided wasm bytes', async () => {
    const moduleBytes = await loadWasmBytes();

    await expect(initWasm({ module_or_path: moduleBytes })).resolves.toBeUndefined();

    const result = compile('cube(1);');
    expect(result.nodeCount).toBeGreaterThanOrEqual(0);
  });

  /**
   * Verifies that repeated initialization attempts do not throw.
   *
   * @example
   * await initWasm();
   * await initWasm();
   */
  it('is idempotent once the wasm bundle is ready', async () => {
    await initWasm();
    await expect(initWasm()).resolves.toBeUndefined();

    const result = compile('cube(1);');
    expect(result.nodeCount).toBeGreaterThanOrEqual(0);
  });
});

/**
 * Groups compile helper scenarios requiring prior initialization.
 *
 * @example
 * describe('compile', () => {
 *   // wasm assertions
 * });
 */
describe('compile', () => {
  /**
   * Confirms an explicit error is thrown when initWasm was not called.
   *
   * @example
   * expect(() => compile('cube();')).toThrowError('WASM not initialized');
   */
  it('throws when called before init', () => {
    expect(() => compile('cube();')).toThrow('WASM not initialized');
  });

  /**
   * Confirms wasm entry point is invoked for initialized scenarios with default artifacts.
   *
   * @example
   * await initWasm();
   * const result = compile('cube();');
   */
  it('provides node counts after default initialization', async () => {
    await initWasm();

    const result = compile('cube(1);');

    expect(result.nodeCount).toBeGreaterThanOrEqual(0);
  });
});
