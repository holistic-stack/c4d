# Lesson Learned – WASM libc stubs limited to wasm32 builds

## Context
- Date: 2025-11-22
- Vertical Slice: Diagnostic transfer from Rust WASM to Playground (plain-object `DiagnosticData`).
- Impacted Areas: `libs/wasm` build pipeline, libc shim (`src/libc_stubs.c`), `build-wasm.js` tooling, Playground worker/UI diagnostics handling.

## Issue Summary
Running `node build-wasm.js` failed with
```
src/libc_stubs.c:6:5: error: conflicting types for 'fprintf'
src/libc_stubs.c:9:5: error: conflicting types for 'vfprintf'
```
`libc_stubs.c` declared `fprintf`/`vfprintf` parameters as `void *`, but the generated `stdio.h` expected `FILE *` / specific varargs signatures. Because the stubs were compiled even for host targets, the conflict surfaced during wasm builds and blocked diagnostics validation.

## Root Cause
- The libc stub function prototypes drifted from the upstream `stdio.h` definitions.
- `build.rs` compiled `src/libc_stubs.c` whenever `TARGET` contained `wasm32`, but host-side `cargo build` invocations (e.g., during `node build-wasm.js`) still saw the conflicting prototypes before reaching wasm32-specific code paths.

## Resolution
1. Align stub signatures with the expected prototypes:
   ```c
   int fprintf(FILE *stream, const char *format, ...);
   int vfprintf(FILE *stream, const char *format, void *ap);
   ```
2. Re-run `node build-wasm.js` to regenerate `libs/wasm/pkg`. Build succeeded after the signature fix.
3. Run `pnpm check` in `apps/playground` to validate the `DiagnosticData` TypeScript plumbing and confirm no Svelte/TS errors.

## Verification
- `node build-wasm.js` now completes successfully, producing the wasm-bindgen bundle.
- `pnpm check` passes with zero diagnostics, ensuring the new plain-object diagnostic flow is type-safe in the Playground.

## Action Items / Prevention
- Keep libc stub prototypes synchronized with the generated headers; consider adding CI checks that compile `libc_stubs.c` with `-Wall -Werror` against the wasm include directory.
- Document (here) that any future changes to libc stubs must mirror the canonical `stdio.h` signatures to avoid redeclaration conflicts.
- Prefer leveraging Rust-side shims or `cfg(target_arch = "wasm32")` modules where possible to minimize raw C stubs.
