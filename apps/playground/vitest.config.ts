/**
 * Vitest configuration dedicated to unit tests that do not rely on the Vite dev server.
 *
 * @example
 * // Run all tests without pulling in the SvelteKit Vite plugin:
 * // pnpm vitest run
 */
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';

/**
 * Absolute path to the project root used to resolve shared aliases.
 *
 * @example
 * console.log(projectRoot);
 */
const projectRoot = fileURLToPath(new URL('.', import.meta.url));

/**
 * Canonical alias mapping reused by both Vite resolve and Vitest `test.alias` settings.
 *
 * @example
 * console.log(aliasMap.$lib);
 */
const aliasMap = {
  /**
   * Entry point for wasm-bindgen outputs shared between runtime and tests.
   *
   * @example
   * import init from '$wasm/wasm.js';
   */
  $wasm: resolve(projectRoot, '../../libs/wasm/pkg'),
  /**
   * SvelteKit's `$lib` alias mirrored for Vitest to keep imports consistent.
   *
   * @example
   * import { wasmNotInitializedMessage } from '$lib/constants/app-config';
   */
  $lib: resolve(projectRoot, 'src/lib')
} as const;

export default defineConfig({
  /**
   * Align Vitest's module resolution with Vite so imports such as `$wasm/wasm.js`
   * work both in-app and inside isolated unit tests.
   */
  resolve: {
    alias: aliasMap
  },
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts'],
    /**
     * Mirrors project aliases so Vitest's module graph matches runtime behavior.
     */
    alias: aliasMap
  }
});
