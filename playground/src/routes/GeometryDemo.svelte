<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { loadWasm } from '$lib/wasm/wasm-loader';

	let wasmLoading = false;
	let wasmError: string | null = null;
	let cubeVertices: number[] | null = null;
	let cubeInfo: string | null = null;

	async function createCube() {
		if (!browser || wasmLoading) return;

		wasmLoading = true;
		wasmError = null;

		try {
			const wasm = await loadWasm();
			
			// Create cube parameters
			const params = new wasm.CubeParams(0, 0, 0, 2, 2, 2); // Center at origin, size 2x2x2
			
			// Get cube vertices
			const vertices = wasm.get_cube_vertices(params);
			cubeVertices = Array.from(vertices);
			
			// Get cube info
			const info = wasm.create_cube_mesh(params);
			cubeInfo = info;
			
		} catch (error) {
			const err = error as Error;
			wasmError = err.message ?? String(error);
		} finally {
			wasmLoading = false;
		}
	}

	onMount(() => {
		void createCube();
	});

	function formatVertices(vertices: number[]): string {
		const vertexCount = vertices.length / 3;
		let result = `Vertices (${vertexCount}):\n`;
		for (let i = 0; i < Math.min(5, vertexCount); i++) {
			const x = vertices[i * 3];
			const y = vertices[i * 3 + 1];
			const z = vertices[i * 3 + 2];
			result += `  ${i}: (${x.toFixed(2)}, ${y.toFixed(2)}, ${z.toFixed(2)})\n`;
		}
		if (vertexCount > 5) {
			result += `  ... and ${vertexCount - 5} more`;
		}
		return result;
	}
</script>

<div class="geometry-demo">
	<h2>Manifold Geometry Demo</h2>
	
	<button on:click={createCube} disabled={wasmLoading}>
		{#if wasmLoading}
			Creating...
		{:else}
			Create Cube
		{/if}
	</button>

	{#if cubeInfo}
		<div class="info-box">
			<h3>Cube Information</h3>
			<p>{cubeInfo}</p>
		</div>
	{/if}

	{#if cubeVertices && cubeVertices.length > 0}
		<div class="vertices-box">
			<h3>Vertex Data</h3>
			<pre>{formatVertices(cubeVertices)}</pre>
		</div>
	{/if}

	{#if wasmError}
		<div class="error-box">
			<h3>Error</h3>
			<p>{wasmError}</p>
		</div>
	{/if}
</div>

<style>
	.geometry-demo {
		margin: 2rem 0;
		padding: 1.5rem;
		border: 1px solid #ccc;
		border-radius: 8px;
		background: #f9f9f9;
	}

	.info-box, .vertices-box, .error-box {
		margin-top: 1rem;
		padding: 1rem;
		border-radius: 4px;
	}

	.info-box {
		background: #e8f5e8;
		border: 1px solid #4caf50;
	}

	.vertices-box {
		background: #f0f8ff;
		border: 1px solid #2196f3;
	}

	.error-box {
		background: #ffebee;
		border: 1px solid #f44336;
	}

	pre {
		margin: 0;
		font-family: 'Courier New', monospace;
		font-size: 0.9rem;
		white-space: pre-wrap;
	}

	button {
		padding: 0.5rem 1rem;
		background: #1976d2;
		color: white;
		border: none;
		border-radius: 4px;
		cursor: pointer;
		font-size: 1rem;
	}

	button:hover:not(:disabled) {
		background: #1565c0;
	}

	button:disabled {
		background: #ccc;
		cursor: not-allowed;
	}
</style>