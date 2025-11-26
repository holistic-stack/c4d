/**
 * # OpenSCAD Web Worker
 *
 * Handles WASM compilation in a background thread to keep the UI responsive.
 * Communicates with the main thread via postMessage/onmessage.
 *
 * ## Architecture
 *
 * ```
 * Main Thread                    Worker Thread
 * ─────────────                  ─────────────
 * postMessage(COMPILE) ────────► onmessage
 *                                  │
 *                                  ▼
 *                               Parse CST
 *                                  │
 *                                  ▼
 *                               render_from_cst()
 *                                  │
 * onmessage ◄──────────────────── postMessage(RESULT)
 * ```
 *
 * ## Message Types
 *
 * - `INITIALIZE`: Initialize WASM and parser modules
 * - `COMPILE`: Compile OpenSCAD source to mesh
 * - `INITIALIZED`: Worker is ready
 * - `RESULT`: Compilation result with mesh data
 * - `ERROR`: Compilation error with diagnostics
 */

import init, * as wasmExports from '../wasm/pkg/openscad_wasm.js';
import { initParser, parseOpenSCAD, serializeTree } from '../parser/openscad-parser.js';

/** Message types from main thread to worker */
interface WorkerRequest {
	type: 'INITIALIZE' | 'COMPILE';
	id?: number;
	source?: string;
}

/** Message types from worker to main thread */
interface WorkerResponse {
	type: 'INITIALIZED' | 'RESULT' | 'ERROR';
	id?: number;
	vertices?: Float32Array;
	indices?: Uint32Array;
	normals?: Float32Array;
	colors?: Float32Array;
	vertexCount?: number;
	triangleCount?: number;
	compileTime?: number;
	error?: string;
	diagnostics?: Array<{
		severity: string;
		message: string;
		start: number;
		end: number;
	}>;
}

/** Track initialization state */
let isInitialized = false;

/**
 * Initializes WASM and parser modules.
 * Must be called before any compilation requests.
 */
async function initialize(): Promise<void> {
	try {
		// Initialize WASM module
		await init();

		// Initialize tree-sitter parser
		await initParser();

		isInitialized = true;

		const response: WorkerResponse = { type: 'INITIALIZED' };
		self.postMessage(response);
	} catch (error) {
		const response: WorkerResponse = {
			type: 'ERROR',
			error: `Initialization failed: ${error instanceof Error ? error.message : String(error)}`
		};
		self.postMessage(response);
	}
}

/**
 * Compiles OpenSCAD source code to mesh data.
 *
 * @param id - Request ID for matching responses
 * @param source - OpenSCAD source code
 */
async function compile(id: number, source: string): Promise<void> {
	const startTime = performance.now();

	try {
		// Parse with web-tree-sitter
		const parseResult = parseOpenSCAD(source);

		// Check for syntax errors
		if (parseResult.errors.length > 0) {
			const response: WorkerResponse = {
				type: 'ERROR',
				id,
				diagnostics: parseResult.errors.map((err) => ({
					severity: 'error',
					message: err.message,
					start: err.startIndex,
					end: err.endIndex
				}))
			};
			self.postMessage(response);
			return;
		}

		// Serialize CST to JSON
		const cst = serializeTree(parseResult.tree);
		const cstJson = JSON.stringify(cst);

		// Render mesh from CST
		const mesh = wasmExports.render_from_cst(cstJson);
		const compileTime = performance.now() - startTime;

		if (mesh.is_empty()) {
			const response: WorkerResponse = {
				type: 'RESULT',
				id,
				vertexCount: 0,
				triangleCount: 0,
				compileTime
			};
			self.postMessage(response);
			return;
		}

		// Extract mesh data
		const vertices = mesh.vertices();
		const indices = mesh.indices();
		const normals = mesh.has_normals() ? mesh.normals() : undefined;
		const colors = mesh.has_colors() ? mesh.colors() : undefined;

		// Send result with transferable arrays for performance
		const response: WorkerResponse = {
			type: 'RESULT',
			id,
			vertices,
			indices,
			normals,
			colors,
			vertexCount: mesh.vertex_count,
			triangleCount: mesh.triangle_count,
			compileTime
		};

		// Transfer ownership of ArrayBuffers for zero-copy performance
		const transferList: ArrayBuffer[] = [vertices.buffer, indices.buffer];
		if (normals) transferList.push(normals.buffer);
		if (colors) transferList.push(colors.buffer);

		self.postMessage(response, transferList);
	} catch (error: unknown) {
		const compileTime = performance.now() - startTime;

		// Check if it's a compile error with diagnostics
		if (
			error &&
			typeof error === 'object' &&
			'diagnostics' in error &&
			Array.isArray((error as { diagnostics: unknown }).diagnostics)
		) {
			const response: WorkerResponse = {
				type: 'ERROR',
				id,
				diagnostics: (error as { diagnostics: Array<{ severity: string; message: string; start: number; end: number }> }).diagnostics,
				compileTime
			};
			self.postMessage(response);
		} else {
			const response: WorkerResponse = {
				type: 'ERROR',
				id,
				error: error instanceof Error ? error.message : String(error),
				compileTime
			};
			self.postMessage(response);
		}
	}
}

/**
 * Handle messages from main thread.
 */
self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
	const { type, id, source } = event.data;

	switch (type) {
		case 'INITIALIZE':
			await initialize();
			break;

		case 'COMPILE':
			if (!isInitialized) {
				const response: WorkerResponse = {
					type: 'ERROR',
					id,
					error: 'Worker not initialized. Call INITIALIZE first.'
				};
				self.postMessage(response);
				return;
			}

			if (typeof id !== 'number' || typeof source !== 'string') {
				const response: WorkerResponse = {
					type: 'ERROR',
					id,
					error: 'Invalid COMPILE request: missing id or source'
				};
				self.postMessage(response);
				return;
			}

			await compile(id, source);
			break;

		default:
			const response: WorkerResponse = {
				type: 'ERROR',
				error: `Unknown message type: ${type}`
			};
			self.postMessage(response);
	}
};

// Auto-initialize on worker start
initialize();
