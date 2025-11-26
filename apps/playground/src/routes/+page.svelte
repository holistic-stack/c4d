<script lang="ts">
	/**
	 * # OpenSCAD Playground
	 *
	 * Main page component for the OpenSCAD playground.
	 * Provides a code editor and 3D viewer for OpenSCAD models.
	 * 
	 * ## Architecture
	 * 
	 * Uses Web Worker for non-blocking UI:
	 * 1. Main thread sends source code to worker
	 * 2. Worker: web-tree-sitter parses OpenSCAD source to CST
	 * 3. Worker: Rust WASM converts CST to AST, evaluates, and generates mesh
	 * 4. Worker sends mesh data back via transferable ArrayBuffers
	 * 5. Main thread: Three.js renders the mesh
	 *
	 * ## Coordinate System
	 *
	 * Uses Z-up axis convention to match OpenSCAD (CAD/engineering standard).
	 */
	import { onMount, onDestroy } from 'svelte';
	import { SceneManager } from '$lib/viewer/scene-manager';
	import { WorkerClient, type Diagnostic, type CompileError } from '$lib/worker/worker-client';

	/** Default OpenSCAD code to show on load */
	const DEFAULT_CODE = `// OpenSCAD Playground
// Try editing this code!

// Boolean operations demo
translate([-24, 0, 0]) {
    union() {
        cube(15, center=true);
        sphere(10);
    }
}

intersection() {
    cube(15, center=true);
    sphere(10);
}

translate([24, 0, 0]) {
    difference() {
        cube(15, center=true);
        sphere(10);
    }
}`;

	/** Current OpenSCAD source code */
	let code = $state(DEFAULT_CODE);

	/** Compilation status */
	let status = $state<'idle' | 'compiling' | 'success' | 'error'>('idle');

	/** Error diagnostics */
	let diagnostics = $state<Diagnostic[]>([]);

	/** Mesh statistics */
	let meshStats = $state<{ vertices: number; triangles: number } | null>(null);

	/** Compilation time in milliseconds */
	let compileTime = $state<number | null>(null);

	/** Canvas element reference */
	let canvasElement: HTMLCanvasElement | null = $state(null);

	/** Scene manager instance */
	let sceneManager: SceneManager | null = null;

	/** Web Worker client for non-blocking compilation */
	let workerClient: WorkerClient | null = null;

	/** Debounce timer for auto-compile */
	let compileTimer: ReturnType<typeof setTimeout> | null = null;

	/**
	 * Compiles the current code and updates the mesh.
	 * 
	 * Pipeline (via Web Worker for non-blocking UI):
	 * 1. Send source to worker
	 * 2. Worker parses CST and calls WASM
	 * 3. Worker returns mesh data via transferable ArrayBuffers
	 * 4. Update Three.js scene on main thread
	 */
	async function compile(): Promise<void> {
		if (!workerClient || !workerClient.isInitialized()) {
			status = 'error';
			diagnostics = [{ severity: 'error', message: 'Worker not initialized', start: 0, end: 0 }];
			return;
		}

		status = 'compiling';
		diagnostics = [];

		try {
			// Compile via Web Worker (non-blocking)
			const result = await workerClient.compile(code);

			compileTime = result.compileTime;

			if (result.vertexCount === 0) {
				status = 'success';
				meshStats = { vertices: 0, triangles: 0 };
				sceneManager?.clearMesh();
				return;
			}

			// Update mesh in scene with transferred ArrayBuffers
			sceneManager?.updateMesh(
				result.vertices,
				result.indices,
				result.normals,
				result.colors
			);

			meshStats = {
				vertices: result.vertexCount,
				triangles: result.triangleCount
			};
			status = 'success';
		} catch (error: unknown) {
			status = 'error';

			// Handle CompileError with diagnostics
			const compileError = error as CompileError;
			if (compileError.diagnostics) {
				diagnostics = compileError.diagnostics;
				compileTime = compileError.compileTime ?? null;
			} else {
				console.error('Compilation error:', error);
				diagnostics = [{ 
					severity: 'error', 
					message: compileError.message || String(error), 
					start: 0, 
					end: 0
				}];
			}
		}
	}

	/**
	 * Handles code changes with debounced auto-compile.
	 */
	function handleCodeChange(event: Event): void {
		const target = event.target as HTMLTextAreaElement;
		code = target.value;

		// Debounce compilation
		if (compileTimer) {
			clearTimeout(compileTimer);
		}
		compileTimer = setTimeout(() => {
			compile();
		}, 500);
	}

	/**
	 * Handles window resize events.
	 */
	function handleResize(): void {
		if (canvasElement && sceneManager) {
			const rect = canvasElement.getBoundingClientRect();
			sceneManager.resize(rect.width, rect.height);
		}
	}

	onMount(async () => {
		// Initialize Web Worker (handles WASM and parser internally)
		try {
			workerClient = new WorkerClient();
			await workerClient.initialize();
		} catch (error) {
			console.error('Failed to initialize worker:', error);
			status = 'error';
			diagnostics = [{ 
				severity: 'error', 
				message: `Failed to initialize: ${error instanceof Error ? error.message : String(error)}`,
				start: 0, 
				end: 0
			}];
			return;
		}

		// Initialize scene manager with Z-up axis
		if (canvasElement) {
			sceneManager = new SceneManager();
			sceneManager.attach(canvasElement);
		}

		// Add resize listener
		window.addEventListener('resize', handleResize);

		// Initial compile
		await compile();
	});

	onDestroy(() => {
		// Cleanup
		if (compileTimer) {
			clearTimeout(compileTimer);
		}
		window.removeEventListener('resize', handleResize);
		sceneManager?.dispose();
		workerClient?.dispose();
	});
</script>

<svelte:head>
	<title>OpenSCAD Playground</title>
</svelte:head>

<div class="playground">
	<header class="header">
		<h1>OpenSCAD Playground</h1>
		<div class="status">
			{#if status === 'compiling'}
				<span class="status-compiling">Compiling...</span>
			{:else if status === 'success'}
				<span class="status-success">
					✓ {meshStats?.vertices ?? 0} vertices, {meshStats?.triangles ?? 0} triangles
					{#if compileTime !== null}
						({compileTime.toFixed(1)}ms)
					{/if}
				</span>
			{:else if status === 'error'}
				<span class="status-error">✗ Error</span>
			{:else}
				<span class="status-idle">Ready</span>
			{/if}
		</div>
		<button class="compile-btn" onclick={() => compile()} disabled={status === 'compiling'}>
			Compile
		</button>
	</header>

	<main class="main">
		<section class="editor-panel">
			<textarea
				class="code-editor"
				value={code}
				oninput={handleCodeChange}
				spellcheck="false"
				placeholder="Enter OpenSCAD code..."
			></textarea>

			{#if diagnostics.length > 0}
				<div class="diagnostics">
					<h3>Errors</h3>
					<ul>
						{#each diagnostics as diag}
							<li class="diagnostic-item">
								<span class="diagnostic-severity">{diag.severity}</span>
								<span class="diagnostic-message">{diag.message}</span>
							</li>
						{/each}
					</ul>
				</div>
			{/if}
		</section>

		<section class="viewer-panel">
			<canvas bind:this={canvasElement} class="viewer-canvas"></canvas>
		</section>
	</main>
</div>

<style>
	.playground {
		display: flex;
		flex-direction: column;
		height: 100vh;
		background: #0d0d1a;
		color: #e0e0e0;
		font-family: system-ui, -apple-system, sans-serif;
	}

	.header {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 0.75rem 1rem;
		background: #1a1a2e;
		border-bottom: 1px solid #2a2a4a;
	}

	.header h1 {
		margin: 0;
		font-size: 1.25rem;
		font-weight: 600;
	}

	.status {
		flex: 1;
		font-size: 0.875rem;
	}

	.status-compiling {
		color: #ffd700;
	}

	.status-success {
		color: #4ade80;
	}

	.status-error {
		color: #f87171;
	}

	.status-idle {
		color: #888;
	}

	.compile-btn {
		padding: 0.5rem 1rem;
		background: #3b82f6;
		color: white;
		border: none;
		border-radius: 4px;
		cursor: pointer;
		font-size: 0.875rem;
	}

	.compile-btn:hover:not(:disabled) {
		background: #2563eb;
	}

	.compile-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.main {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	.editor-panel {
		display: flex;
		flex-direction: column;
		width: 40%;
		min-width: 300px;
		border-right: 1px solid #2a2a4a;
	}

	.code-editor {
		flex: 1;
		padding: 1rem;
		background: #0d0d1a;
		color: #e0e0e0;
		border: none;
		resize: none;
		font-family: 'Fira Code', 'Consolas', monospace;
		font-size: 0.875rem;
		line-height: 1.5;
		tab-size: 4;
	}

	.code-editor:focus {
		outline: none;
	}

	.diagnostics {
		padding: 0.75rem 1rem;
		background: #1a0a0a;
		border-top: 1px solid #4a2a2a;
		max-height: 150px;
		overflow-y: auto;
	}

	.diagnostics h3 {
		margin: 0 0 0.5rem;
		font-size: 0.875rem;
		color: #f87171;
	}

	.diagnostics ul {
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.diagnostic-item {
		padding: 0.25rem 0;
		font-size: 0.8125rem;
	}

	.diagnostic-severity {
		color: #f87171;
		margin-right: 0.5rem;
	}

	.diagnostic-message {
		color: #e0e0e0;
	}

	.viewer-panel {
		flex: 1;
		position: relative;
	}

	.viewer-canvas {
		width: 100%;
		height: 100%;
		display: block;
	}
</style>
