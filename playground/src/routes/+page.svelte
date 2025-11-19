<script lang="ts">
	import Counter from './Counter.svelte';
	import GeometryDemo from './GeometryDemo.svelte';
	import welcome from '$lib/images/svelte-welcome.webp';
	import welcomeFallback from '$lib/images/svelte-welcome.png';
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { helloFromWasm } from '$lib/wasm/wasm-loader';

	let wasmMessage: string | null = null;
	let wasmError: string | null = null;
	let wasmLoading = false;

	async function runWasmHello() {
		if (!browser || wasmLoading) return;

		wasmLoading = true;
		wasmError = null;

		try {
			wasmMessage = await helloFromWasm();
		} catch (error) {
			const err = error as Error;
			wasmError = err.message ?? String(error);
		} finally {
			wasmLoading = false;
		}
	}

	onMount(() => {
		void runWasmHello();
	});
</script>

<svelte:head>
	<title>Rust OpenSCAD Playground</title>
	<meta name="description" content="Interactive playground for Rust OpenSCAD geometry kernel" />
</svelte:head>

<section>
	<h1>
		Rust OpenSCAD Playground
	</h1>

	<h2>
		Interactive geometry kernel with WebAssembly
	</h2>

	<Counter />

	<GeometryDemo />

	<div class="wasm-demo">
		<h3>WASM hello world</h3>
		<button on:click={runWasmHello} disabled={wasmLoading}>
			{#if wasmLoading}
				Running...
			{:else}
				Run hello_world() from WASM
			{/if}
		</button>

		{#if wasmMessage}
			<p class="wasm-message">{wasmMessage}</p>
		{/if}

		{#if wasmError}
			<p class="wasm-error">Error: {wasmError}</p>
		{/if}
	</div>
</section>

<style>
	section {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		flex: 0.6;
	}

	h1 {
		width: 100%;
	}

	.wasm-demo {
		margin-top: 2rem;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.75rem;
	}

	.wasm-message {
		font-weight: 600;
	}

	.wasm-error {
		color: #e53935;
	}
</style>
