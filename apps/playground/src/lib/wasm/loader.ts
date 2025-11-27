/**
 * # WASM Loader Module
 *
 * Loads and initializes the OpenSCAD WASM module.
 *
 * ## Architecture
 *
 * ```text
 * JavaScript → render(source) → Rust WASM (full pipeline) → Mesh Data
 * ```
 *
 * ## Usage
 *
 * ```typescript
 * import { initWasm, render, getVersion } from './lib/wasm/loader';
 *
 * await initWasm();
 * const result = render('cube(10);');
 * ```
 */

// =============================================================================
// TYPES
// =============================================================================

/**
 * Render result from WASM.
 *
 * Contains mesh data as typed arrays for efficient Three.js integration.
 */
export interface RenderResult {
  /** Whether rendering succeeded */
  success: boolean;

  /** Error message if failed */
  error?: string;

  /** Vertex positions (x, y, z) */
  vertices?: Float32Array;

  /** Triangle indices */
  indices?: Uint32Array;

  /** Vertex normals (x, y, z) */
  normals?: Float32Array;

  /** Vertex count */
  vertexCount: number;

  /** Triangle count */
  triangleCount: number;

  /** Render time in milliseconds */
  renderTimeMs: number;
}

/**
 * Raw WASM module interface from wasm-bindgen.
 */
interface WasmModule {
  /** Get module version */
  get_version: () => string;

  /** Render from source code (full pipeline) */
  render: (source: string) => RenderResult;
}

/**
 * WASM init function type.
 */
type WasmInit = () => Promise<unknown>;

// =============================================================================
// MODULE STATE
// =============================================================================

/** WASM module instance */
let wasmModule: WasmModule | null = null;

/** Initialization promise */
let initPromise: Promise<void> | null = null;

// =============================================================================
// INITIALIZATION
// =============================================================================

/**
 * Initialize the WASM module.
 *
 * Loads and instantiates the WASM binary.
 * Safe to call multiple times - will only initialize once.
 *
 * @throws Error if initialization fails
 *
 * @example
 * ```typescript
 * await initWasm();
 * console.log('WASM ready!');
 * ```
 */
export async function initWasm(): Promise<void> {
  if (initPromise) {
    return initPromise;
  }

  if (wasmModule) {
    return Promise.resolve();
  }

  initPromise = doInit();
  return initPromise;
}

/**
 * Perform actual WASM initialization.
 *
 * @internal
 */
async function doInit(): Promise<void> {
  try {
    console.log('[WASM] Loading module...');

    // Dynamic import of WASM bindings
    const wasm = await import('./pkg/openscad_wasm.js');

    // Initialize WASM module
    const init = wasm.default as WasmInit;
    await init();

    // Store module reference
    wasmModule = wasm as unknown as WasmModule;

    console.log(`[WASM] Loaded version ${wasmModule.get_version()}`);
  } catch (error) {
    initPromise = null;
    const message = error instanceof Error ? error.message : 'Unknown error';
    throw new Error(`Failed to initialize WASM: ${message}`);
  }
}

// =============================================================================
// PUBLIC API
// =============================================================================

/**
 * Check if WASM is initialized and ready.
 *
 * @returns true if WASM is ready to use
 */
export function isWasmReady(): boolean {
  return wasmModule !== null;
}

/**
 * Get the WASM module version.
 *
 * @returns Version string
 * @throws Error if WASM not initialized
 *
 * @example
 * ```typescript
 * const version = getVersion();
 * console.log(`WASM v${version}`);
 * ```
 */
export function getVersion(): string {
  if (!wasmModule) {
    throw new Error('WASM not initialized. Call initWasm() first.');
  }
  return wasmModule.get_version();
}

/**
 * Render OpenSCAD source code.
 *
 * Full pipeline: source → parser → AST → evaluator → mesh.
 * All processing done in pure Rust WASM.
 *
 * @param source - OpenSCAD source code
 * @returns Render result with mesh data
 * @throws Error if WASM not initialized
 *
 * @example
 * ```typescript
 * const result = render('cube(10);');
 * if (result.success) {
 *   scene.updateMesh(result.vertices, result.indices, result.normals);
 * }
 * ```
 */
export function render(source: string): RenderResult {
  if (!wasmModule) {
    throw new Error('WASM not initialized. Call initWasm() first.');
  }

  try {
    return wasmModule.render(source);
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown WASM error',
      vertexCount: 0,
      triangleCount: 0,
      renderTimeMs: 0,
    };
  }
}
