/**
 * # WASM Loader
 *
 * Handles loading and initialization of the OpenSCAD WASM module.
 * Provides a singleton pattern for module access.
 *
 * ## Usage
 *
 * ```typescript
 * import { initWasm, getWasm } from '$lib/wasm/loader';
 * import { initParser, parseOpenSCAD, serializeTree } from '$lib/parser/openscad-parser';
 *
 * await initWasm();
 * await initParser();
 * 
 * const wasm = getWasm();
 * const { tree } = parseOpenSCAD("cube(10);");
 * const cst = serializeTree(tree);
 * const mesh = wasm.render_from_cst(JSON.stringify(cst));
 * ```
 */

import type { OpenSCADWasm } from './types';
// Import the WASM module directly - Vite handles the bundling
import initWasmModule, * as wasmExports from './pkg/openscad_wasm.js';

/** Cached WASM module instance */
let wasmModule: OpenSCADWasm | null = null;

/** Promise for ongoing initialization */
let initPromise: Promise<OpenSCADWasm> | null = null;

/**
 * Initializes the WASM module.
 *
 * This function is idempotent - calling it multiple times will return
 * the same module instance.
 *
 * @returns Promise resolving to the WASM module
 * @throws Error if WASM loading fails
 *
 * @example
 * ```typescript
 * const wasm = await initWasm();
 * // Use with web-tree-sitter parser
 * ```
 */
export async function initWasm(): Promise<OpenSCADWasm> {
	// Return cached module if already initialized
	if (wasmModule) {
		return wasmModule;
	}

	// Return ongoing initialization if in progress
	if (initPromise) {
		return initPromise;
	}

	// Start initialization
	initPromise = loadWasmModule();

	try {
		wasmModule = await initPromise;
		return wasmModule;
	} finally {
		initPromise = null;
	}
}

/**
 * Loads the WASM module using the bundled wasm-bindgen output.
 */
async function loadWasmModule(): Promise<OpenSCADWasm> {
	try {
		// Initialize the WASM module
		await initWasmModule();
		
		// Return the exports as our WASM interface
		return wasmExports as unknown as OpenSCADWasm;
	} catch (error) {
		// If WASM is not available, throw a descriptive error
		throw new Error(
			`Failed to load WASM module. ` +
			`Make sure to build the WASM package first with 'node scripts/build-wasm.js'. ` +
			`Original error: ${error instanceof Error ? error.message : String(error)}`
		);
	}
}

/**
 * Gets the initialized WASM module.
 *
 * @returns The WASM module
 * @throws Error if the module has not been initialized
 *
 * @example
 * ```typescript
 * await initWasm();
 * const wasm = getWasm();
 * ```
 */
export function getWasm(): OpenSCADWasm {
	if (!wasmModule) {
		throw new Error(
			'WASM module not initialized. Call initWasm() first.'
		);
	}
	return wasmModule;
}

/**
 * Checks if the WASM module is initialized.
 *
 * @returns True if the module is ready
 */
export function isWasmReady(): boolean {
	return wasmModule !== null;
}
