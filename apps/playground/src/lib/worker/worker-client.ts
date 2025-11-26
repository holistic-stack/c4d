/**
 * # Worker Client
 *
 * Promise-based wrapper for communicating with the OpenSCAD Web Worker.
 * Provides a clean async API for compilation requests.
 *
 * ## Usage
 *
 * ```typescript
 * const client = new WorkerClient();
 * await client.initialize();
 *
 * const result = await client.compile('cube(10);');
 * console.log(result.vertices, result.indices);
 *
 * client.dispose();
 * ```
 */

/** Diagnostic information for errors */
export interface Diagnostic {
	severity: string;
	message: string;
	start: number;
	end: number;
}

/** Successful compilation result */
export interface CompileResult {
	vertices: Float32Array;
	indices: Uint32Array;
	normals?: Float32Array;
	colors?: Float32Array;
	vertexCount: number;
	triangleCount: number;
	compileTime: number;
}

/** Compilation error with diagnostics */
export interface CompileError {
	message: string;
	diagnostics?: Diagnostic[];
	compileTime?: number;
}

/** Pending request tracker */
interface PendingRequest {
	resolve: (result: CompileResult) => void;
	reject: (error: CompileError) => void;
}

/**
 * Promise-based client for the OpenSCAD Web Worker.
 *
 * Handles:
 * - Worker lifecycle management
 * - Request/response correlation via IDs
 * - Promise-based async API
 * - Error handling and diagnostics
 */
export class WorkerClient {
	private worker: Worker | null = null;
	private requestId = 0;
	private pendingRequests = new Map<number, PendingRequest>();
	private initializePromise: Promise<void> | null = null;
	private isReady = false;

	/**
	 * Creates a new WorkerClient instance.
	 * Does not start the worker - call initialize() first.
	 */
	constructor() {
		// Worker is created lazily in initialize()
	}

	/**
	 * Initializes the worker and WASM modules.
	 * Safe to call multiple times - subsequent calls return the same promise.
	 *
	 * @returns Promise that resolves when worker is ready
	 * @throws Error if worker fails to initialize
	 *
	 * @example
	 * ```typescript
	 * const client = new WorkerClient();
	 * await client.initialize();
	 * ```
	 */
	async initialize(): Promise<void> {
		// Return existing promise if already initializing
		if (this.initializePromise) {
			return this.initializePromise;
		}

		// Return immediately if already ready
		if (this.isReady) {
			return Promise.resolve();
		}

		this.initializePromise = this.doInitialize();
		return this.initializePromise;
	}

	/**
	 * Internal initialization logic.
	 */
	private async doInitialize(): Promise<void> {
		return new Promise((resolve, reject) => {
			// Create worker using Vite's worker import syntax
			this.worker = new Worker(
				new URL('./openscad-worker.ts', import.meta.url),
				{ type: 'module' }
			);

			// Set up message handler
			this.worker.onmessage = (event) => {
				this.handleMessage(event.data);
			};

			// Set up error handler
			this.worker.onerror = (error) => {
				reject(new Error(`Worker error: ${error.message}`));
			};

			// Wait for INITIALIZED message
			const initHandler = (event: MessageEvent) => {
				const { type, error } = event.data;

				if (type === 'INITIALIZED') {
					this.isReady = true;
					this.worker?.removeEventListener('message', initHandler);
					resolve();
				} else if (type === 'ERROR' && !this.isReady) {
					this.worker?.removeEventListener('message', initHandler);
					reject(new Error(error || 'Worker initialization failed'));
				}
			};

			this.worker.addEventListener('message', initHandler);
		});
	}

	/**
	 * Handles messages from the worker.
	 *
	 * @param data - Message data from worker
	 */
	private handleMessage(data: {
		type: string;
		id?: number;
		vertices?: Float32Array;
		indices?: Uint32Array;
		normals?: Float32Array;
		colors?: Float32Array;
		vertexCount?: number;
		triangleCount?: number;
		compileTime?: number;
		error?: string;
		diagnostics?: Diagnostic[];
	}): void {
		const { type, id } = data;

		// Ignore messages without IDs (like INITIALIZED)
		if (typeof id !== 'number') {
			return;
		}

		const pending = this.pendingRequests.get(id);
		if (!pending) {
			console.warn(`No pending request for id ${id}`);
			return;
		}

		this.pendingRequests.delete(id);

		if (type === 'RESULT') {
			pending.resolve({
				vertices: data.vertices || new Float32Array(0),
				indices: data.indices || new Uint32Array(0),
				normals: data.normals,
				colors: data.colors,
				vertexCount: data.vertexCount || 0,
				triangleCount: data.triangleCount || 0,
				compileTime: data.compileTime || 0
			});
		} else if (type === 'ERROR') {
			pending.reject({
				message: data.error || 'Unknown error',
				diagnostics: data.diagnostics,
				compileTime: data.compileTime
			});
		}
	}

	/**
	 * Compiles OpenSCAD source code to mesh data.
	 *
	 * @param source - OpenSCAD source code
	 * @returns Promise resolving to mesh data
	 * @throws CompileError if compilation fails
	 *
	 * @example
	 * ```typescript
	 * try {
	 *   const result = await client.compile('cube(10);');
	 *   console.log(`${result.vertexCount} vertices`);
	 * } catch (error) {
	 *   console.error(error.diagnostics);
	 * }
	 * ```
	 */
	async compile(source: string): Promise<CompileResult> {
		if (!this.isReady || !this.worker) {
			throw new Error('Worker not initialized. Call initialize() first.');
		}

		const id = this.requestId++;

		return new Promise((resolve, reject) => {
			this.pendingRequests.set(id, { resolve, reject });

			this.worker!.postMessage({
				type: 'COMPILE',
				id,
				source
			});
		});
	}

	/**
	 * Checks if the worker is ready for compilation.
	 *
	 * @returns True if worker is initialized and ready
	 */
	isInitialized(): boolean {
		return this.isReady;
	}

	/**
	 * Disposes of the worker and cleans up resources.
	 * Call this when the component is destroyed.
	 */
	dispose(): void {
		// Reject all pending requests
		for (const [id, pending] of this.pendingRequests) {
			pending.reject({ message: 'Worker disposed' });
		}
		this.pendingRequests.clear();

		// Terminate worker
		if (this.worker) {
			this.worker.terminate();
			this.worker = null;
		}

		this.isReady = false;
		this.initializePromise = null;
	}
}
