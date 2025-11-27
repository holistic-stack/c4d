/**
 * # OpenSCAD Playground - Main Entry Point
 *
 * Initializes the application:
 * 1. Loads tree-sitter parser with OpenSCAD grammar
 * 2. Loads WASM module
 * 3. Sets up Three.js scene
 * 4. Connects editor to render pipeline
 *
 * ## Architecture
 *
 * ```text
 * Editor Input → tree-sitter → CST → WASM → Mesh → Three.js
 * ```
 */

import { initParser, parseToJson, isParserReady } from './lib/parser/openscad-parser';
import { initWasm, renderFromCst, getVersion, isWasmReady } from './lib/wasm/loader';
import { SceneManager } from './lib/viewer/scene-manager';

// =============================================================================
// DOM ELEMENTS
// =============================================================================

/** Editor textarea */
const editorElement = document.getElementById('editor') as HTMLTextAreaElement;

/** Render button */
const renderButton = document.getElementById('render-btn') as HTMLButtonElement;

/** Status bar */
const statusBar = document.getElementById('status-bar') as HTMLDivElement;

/** Version display */
const versionElement = document.getElementById('version') as HTMLSpanElement;

/** Loading overlay */
const loadingOverlay = document.getElementById('loading-overlay') as HTMLDivElement;

/** Viewer canvas */
const viewerCanvas = document.getElementById('viewer-canvas') as HTMLCanvasElement;

// =============================================================================
// APPLICATION STATE
// =============================================================================

/** Scene manager instance */
let sceneManager: SceneManager | null = null;

// =============================================================================
// UI HELPERS
// =============================================================================

/**
 * Update status bar with message.
 *
 * @param message - Status message
 * @param isError - Whether this is an error
 */
function setStatus(message: string, isError = false): void {
  statusBar.textContent = message;
  statusBar.classList.toggle('error', isError);
}

/**
 * Enable or disable render button.
 *
 * @param enabled - Whether to enable
 */
function setRenderEnabled(enabled: boolean): void {
  renderButton.disabled = !enabled;
}

/**
 * Hide loading overlay.
 */
function hideLoading(): void {
  loadingOverlay.classList.add('hidden');
}

// =============================================================================
// RENDER PIPELINE
// =============================================================================

/**
 * Render the current editor content.
 *
 * Pipeline: Editor → tree-sitter → CST → WASM → Mesh → Three.js
 */
async function handleRender(): Promise<void> {
  if (!isParserReady() || !isWasmReady() || !sceneManager) {
    setStatus('Not ready', true);
    return;
  }

  const source = editorElement.value;
  setStatus('Parsing...');
  setRenderEnabled(false);

  try {
    const totalStart = performance.now();

    // Step 1: Parse with tree-sitter
    const parseStart = performance.now();
    const cstJson = parseToJson(source);
    const parseTime = performance.now() - parseStart;

    // Step 2: Render with WASM
    setStatus('Rendering...');
    const renderStart = performance.now();
    const result = renderFromCst(cstJson);
    const wasmTime = performance.now() - renderStart;

    const totalTime = performance.now() - totalStart;

    if (result.success && result.vertices && result.indices && result.normals) {
      // Step 3: Update Three.js scene
      sceneManager.updateMesh(result.vertices, result.indices, result.normals);

      setStatus(
        `✓ ${result.vertexCount} vertices, ${result.triangleCount} triangles | ` +
        `Parse: ${parseTime.toFixed(1)}ms, WASM: ${wasmTime.toFixed(1)}ms, ` +
        `Total: ${totalTime.toFixed(1)}ms`
      );
    } else {
      setStatus(`Error: ${result.error ?? 'Unknown error'}`, true);
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Unknown error';
    setStatus(`Error: ${message}`, true);
    console.error('Render error:', error);
  } finally {
    setRenderEnabled(true);
  }
}

// =============================================================================
// INITIALIZATION
// =============================================================================

/**
 * Initialize the application.
 */
async function init(): Promise<void> {
  try {
    setStatus('Loading parser...');

    // Initialize tree-sitter parser
    await initParser();
    console.log('[App] Parser initialized');

    setStatus('Loading WASM...');

    // Initialize WASM module
    await initWasm();
    console.log('[App] WASM initialized');

    // Update version display
    versionElement.textContent = `WASM v${getVersion()}`;

    // Initialize Three.js scene
    sceneManager = new SceneManager(viewerCanvas);
    console.log('[App] Scene initialized');

    // Enable UI
    setRenderEnabled(true);
    hideLoading();
    setStatus('Ready - Click Render or press Ctrl+Enter');

    // Set up event handlers
    renderButton.addEventListener('click', handleRender);

    // Keyboard shortcut: Ctrl+Enter to render
    editorElement.addEventListener('keydown', (event) => {
      if (event.ctrlKey && event.key === 'Enter') {
        event.preventDefault();
        handleRender();
      }
    });

    // Auto-render on load
    handleRender();
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Unknown error';
    setStatus(`Initialization failed: ${message}`, true);
    console.error('Init error:', error);
    hideLoading();
  }
}

// Start application
init();
