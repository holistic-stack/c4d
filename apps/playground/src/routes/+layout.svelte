<script lang="ts">
	/**
	 * # Root Layout
	 *
	 * Root layout component for the OpenSCAD Playground.
	 * Sets up global styles and favicon.
	 */
	import favicon from '$lib/assets/favicon.svg';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children()}

<style>
	/* Global reset styles */
	:global(*) {
		margin: 0;
		padding: 0;
		box-sizing: border-box;
	}

	:global(html, body) {
		height: 100%;
		overflow: hidden;
	}
</style>
