---
description: Build the WASM package with Rayon parallelism support
---

This workflow builds the `libs/wasm` crate for the web target, enabling Rayon parallelism using `wasm-bindgen-rayon`.

## Prerequisites

- Rust Nightly toolchain (required for atomics on WASM)
- `rust-src` component for nightly (`rustup component add rust-src --toolchain nightly`)
- `wasm-pack`

## Build Command

Run the following command in the project root:

```powershell
$env:RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+mutable-globals"; wasm-pack build libs/wasm --target web -- -Z build-std=std,panic_abort
```

## Notes

- `atomics` and `bulk-memory` are required for shared memory parallelism.
- `mutable-globals` is also required by `wasm-bindgen-rayon`.
- `-Z build-std=std,panic_abort` is necessary to recompile the standard library with atomics support for the WASM target.
