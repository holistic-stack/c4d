import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
	plugins: [sveltekit()],
	resolve: {
		alias: {
			// Alias for the WASM package
			'$wasm': path.resolve(__dirname, '../../libs/wasm/pkg'),
			// Provide env shim for WASM C runtime functions
			'env': path.resolve(__dirname, 'src/lib/wasm/pkg/env.js')
		}
	},
	server: {
		fs: {
			// Allow serving files from the libs directory
			allow: [
				path.resolve(__dirname, '../../libs/wasm/pkg'),
				path.resolve(__dirname, 'src'),
				path.resolve(__dirname, 'node_modules'),
				path.resolve(__dirname, '.svelte-kit')
			]
		}
	},
	// Optimize WASM loading
	optimizeDeps: {
		exclude: ['$wasm']
	},
	// Configure WASM handling
	build: {
		target: 'esnext'
	}
});
