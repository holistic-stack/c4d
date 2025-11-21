/**
 * Vitest configuration dedicated to unit tests that do not rely on the Vite dev server.
 *
 * @example
 * // Run all tests without pulling in the SvelteKit Vite plugin:
 * // pnpm vitest run
 */
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts']
  }
});
