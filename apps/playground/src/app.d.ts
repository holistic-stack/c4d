// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		// interface Locals {}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

/**
 * WASM module declaration for the relative import path.
 * This allows TypeScript to understand the dynamic import.
 */
declare module '../../../../libs/wasm/pkg/wasm.js' {
	/** Initialize the WASM module */
	export default function init(): Promise<void>;
	/** Compile OpenSCAD source and return a mesh handle */
	export function compile_and_render(source: string): import('$lib/wasm/types').MeshHandle;
	/** Compile OpenSCAD source and return the node count */
	export function compile_and_count_nodes(source: string): number;
	/** Initialize function (called automatically) */
	export function init(): void;
}

export {};
