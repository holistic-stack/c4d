## Overview
- Add deterministic, reusable tests for the Wasm loader APIs in `playground/src/lib/wasm/wasm-loader.ts:5–20` and `playground/src/lib/wasm/wasm-loader.ts:22–29`.
- Intercept only Wasm fetches and pass through all other requests.
- Use locally built Wasm from `playground/src/lib/wasm/pkg/openscad_wasm_bg.wasm` (produced by `npm run build:wasm`).
- Provide typed test utilities with JSDoc and clear setup/teardown boundaries.

## Files to Add
1. `playground/src/lib/wasm/wasm-loader.test.ts`
   - Comprehensive tests covering success, error paths, memoization, cleanup, and basic cross-env assumptions.
   - Uses the utility functions below.
2. `playground/src/lib/test-utils/wasm-fetch.ts`
   - Reusable, typed fetch interception utilities for Wasm assets only.
   - Exported helpers used by the test and future suites.

## Test Utilities (wasm-fetch.ts)
- `interface WasmUrlMap { readonly [url: string]: string }`
- `interface WasmFetchInterceptorOptions { readonly map?: WasmUrlMap; readonly allowPassThrough?: boolean }`
- `setupWasmFetchInterceptor(options): { restore(): void; calls: number }`
  - Saves the original `globalThis.fetch`.
  - Intercepts if request is a `.wasm` URL (string/URL/Request) or resolves to `file://.../openscad_wasm_bg.wasm` via `import.meta.url` behavior.
  - For matches, returns a `Response` backed by `fs.readFile` for the mapped local path or the resolved file path.
  - For all other requests, calls the original `fetch` untouched.
  - Tracks `calls` for assertions.
- `resolveWasmLocalPath(input): string`
  - Converts `file://` URIs using `node:url.fileURLToPath`.
  - Falls back to explicit map or default `playground/src/lib/wasm/pkg/openscad_wasm_bg.wasm`.
- All utilities documented with JSDoc and exported for reuse; strict TypeScript, no `any`.

## Fetch Interception Setup (per test file)
- `beforeEach`: install interceptor with a map `{ ["openscad_wasm_bg.wasm"]: "playground/src/lib/wasm/pkg/openscad_wasm_bg.wasm" }` and pass-through enabled.
- `afterEach`: restore original fetch, `vi.resetModules()` to clear the loader cache.
- Only intercept when the request targets a `.wasm` URL; non-wasm requests go to original fetch.

## Test Cases (wasm-loader.test.ts)
- Success: `helloFromWasm()` returns `"Hello from Rust WASM!"` (`libs/wasm/src/lib.rs:3–6`).
- Successful module load and type checks:
  - `loadWasm()` resolves to a module exposing `hello_world`.
  - Default initializer is executed once; assert interceptor `calls === 1` across multiple calls (memoization at `playground/src/lib/wasm/wasm-loader.ts:3,6–20`).
- Error: network/instantiation failure
  - Interceptor forces a thrown error for the wasm request; assert `loadWasm()` rejects with that error.
- Error: missing export branch
  - After a successful load, temporarily delete `loaded.hello_world` and assert `helloFromWasm()` throws `"hello_world export not found"` (`playground/src/lib/wasm/wasm-loader.ts:25–29`).
  - Does not mock modules; uses direct property deletion on the real loaded module.
- Cleanup & memory sanity
  - Verify interceptor restoration and `vi.resetModules()` keeps tests isolated.
  - Assert only one wasm fetch across repeated invocations (indirect memory/instance reuse via memoization).
- Cross-env assumptions
  - Assert that the interceptor pass-through works and does not affect non-wasm requests.
  - Document that browser execution is covered by component tests and can be extended with a browser-specific suite.

## Coverage Strategy
- Exercise both exported functions and both branches of `helloFromWasm()` (success + missing export).
- Exercise `loadWasm()` first-load and subsequent cached loads.
- Deterministic error via interceptor-thrown error for wasm request.
- With these, line/branch coverage of `wasm-loader.ts` reaches 100%.

## Cleanup & Determinism
- `afterEach` restores original fetch and resets modules.
- No global state leaks; interceptor returns a `restore()` function.
- Tests use only local files; no network access.

## Cross-Browser Strategy
- Current Vitest browser project includes only `*.svelte.*` tests (`playground/vite.config.ts:19–21`).
- Optional follow-up: add `playground/src/lib/wasm/wasm-loader.browser.svelte.test.ts` that imports `helloFromWasm()` in a minimal Svelte component to validate browser behavior under Playwright.
- This keeps Node and browser coverage separate while sharing the same fetch interceptor utilities.

## Risks & Assumptions
- Assumes `npm run build:wasm` has produced `pkg/openscad_wasm.js` and `openscad_wasm_bg.wasm` before tests run (referenced by `build-wasm.js:14–19`).
- Node’s global `Response`/`fetch` is available (Node 22+), consistent with `playground/package.json` engines and types.
- Deleting `hello_world` on the loaded module is acceptable within the “no mocks except fetch” constraint, as it does not mock imports—only exercises a branch via runtime state.

## Deliverables
- New test file with fully typed, JSDoc-documented utilities and deterministic tests.
- Reusable utility module under `src/lib/test-utils/wasm-fetch.ts` for future test suites.
- Verified 100% line/branch coverage for `wasm-loader.ts`.