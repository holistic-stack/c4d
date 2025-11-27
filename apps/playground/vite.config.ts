/**
 * # Vite Configuration
 *
 * Configures the Vite development server and build for the playground.
 *
 * ## Features
 *
 * - Path aliases (@/, @wasm/)
 * - WASM MIME type headers
 * - WebGL support
 */

import { defineConfig } from 'vite';
import { resolve } from 'path';

export default defineConfig({
  // Path aliases for cleaner imports
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      '@wasm': resolve(__dirname, 'src/lib/wasm'),
    },
  },

  // Development server configuration
  server: {
    port: 5173,
    // Required headers for WASM loading
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },

  // Build configuration
  build: {
    target: 'esnext',
    outDir: 'dist',
  },

  // Optimize dependencies
  optimizeDeps: {
    exclude: ['web-tree-sitter'],
  },
});
