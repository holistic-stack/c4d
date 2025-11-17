export type WasmModule = typeof import('./pkg/openscad_wasm.js');

let wasmModulePromise: Promise<WasmModule> | null = null;

export function loadWasm(): Promise<WasmModule> {
	if (!wasmModulePromise) {
		wasmModulePromise = (async () => {
			const mod = await import('./pkg/openscad_wasm.js');
			const init = (mod as any).default;

			if (typeof init === 'function') {
				await init();
			}

			return mod;
		})();
	}

	return wasmModulePromise;
}

export async function helloFromWasm(): Promise<string> {
	const wasm = await loadWasm();

	if (typeof wasm.hello_world === 'function') {
		return wasm.hello_world();
	}

	throw new Error('hello_world export not found');
}
