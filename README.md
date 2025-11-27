# Rust OpenSCAD Pipeline

Pure Rust reimplementation of the OpenSCAD toolchain with a layered, browser-safe pipeline that parses source text, builds an AST, evaluates geometry, generates meshes, and renders them in WASM for the browser playground.

## Architecture Overview

```
apps/playground  →  libs/wasm  →  libs/openscad-mesh  →  libs/openscad-eval  →  libs/openscad-ast  →  libs/parser
```

Each crate is responsible for a single concern, forming a strict dependency chain (no skipping layers):

| Layer | Crate | Responsibility |
| --- | --- | --- |
| Parser | `libs/parser` | Pure Rust lexer + recursive descent parser producing CST |
| AST | `libs/openscad-ast` | CST → AST transformation plus AST data types |
| Evaluator | `libs/openscad-eval` | AST evaluation into `GeometryNode` IR, variables/functions |
| Mesher | `libs/openscad-mesh` | Primitive meshes, transforms, boolean ops, diagnostics |
| WASM | `libs/wasm` | Browser-safe entry point exposing `render(source)` and diagnostic helpers |
| Playground | `apps/playground` | Three.js viewer, worker integration, diagnostics UI |

## Key Features

- **Pure Rust parsing** – no tree-sitter dependency in WASM build outputs.
- **Visitor-based transformations** – predictable, testable layers (CST→AST, AST→IR, IR→Mesh).
- **OpenSCAD-compatible primitives** – cube, sphere, cylinder, polyhedron, 2D shapes, extrusions.
- **CSG boolean operations** – BSP-based union/difference/intersection with iterative algorithms for WASM safety.
- **Mesh optimization** – vertex deduplication, degenerate triangle removal, edge highlighting metadata.
- **Playground integration** – Z-up Three.js scene, wireframe edges, diagnostic reporting channel.

## Repository Layout

```
├─ apps/
│  └─ playground/        # Vite/Three.js UI that consumes the WASM package
├─ libs/
│  ├─ parser/            # Pure Rust lexer + parser (CST)
│  ├─ openscad-ast/      # AST definitions + CST visitors
│  ├─ openscad-eval/     # Evaluator producing GeometryNode IR
│  ├─ openscad-mesh/     # Mesh builder + CSG operations
│  ├─ wasm/              # wasm-bindgen interface exposing render()
│  └─ openscad-lsp/      # (Placeholder) language server crate
└─ specs/
   └─ pipeline/          # Architecture docs, task lists, detailed plans
```

## Getting Started

### 1. Install Tooling
- Rust toolchain (`rustup`, stable and `wasm32-unknown-unknown` target)
- `wasm-pack` for packaging the WASM crate
- `pnpm` (or npm) for the playground frontend

### 2. Run Tests
```bash
# All workspace tests
cargo test --workspace --lib

# Focused tests
cargo test -p openscad-mesh --lib
```

### 3. Build WASM Package
```bash
wasm-pack build libs/wasm --target web --out-dir ../../apps/playground/src/lib/wasm/pkg
```

### 4. Run the Playground
```bash
cd apps/playground
pnpm install
pnpm dev   # http://localhost:5173
```

## Development Guidelines

- **TDD first**: add or update tests in the relevant crate before changing implementation.
- **SRP & visitor pattern**: keep large visitors split into dedicated modules (expressions.rs, statements.rs, etc.).
- **No mocks (except I/O)**: prefer real components with minimal scaffolding.
- **Strict error handling**: bubble up descriptive errors with `thiserror` enums; avoid silent fallbacks.
- **File size limit**: keep each source file under ~500 lines; split modules when necessary.
- **Browser safety**: avoid `std::fs`/`std::process` in shared crates; guard native-only code behind features.

## Diagnostics & Debugging

- `libs/wasm` exposes helper functions for transferring diagnostics to JS (see `Diagnostic::to_js_object`).
- Playground worker and UI consume plain JS objects shaped by the `DiagnosticData` TypeScript interface.
- Mesh builders emit warnings for invalid geometry (e.g., degenerate faces) that surface as diagnostics.

## Roadmap Snapshot

1. **Parser/AST** ✅ – Pure Rust parser with CST→AST visitors.
2. **Evaluator** ✅ – GeometryNode IR with variables, functions, special vars.
3. **Mesh Builder** ✅ – Core primitives, transforms, boolean ops, mesh optimizations.
4. **WASM Integration** ✅ – `render(source)` pipeline with typed array results.
5. **Playground Enhancements** 🚧 – Full diagnostics, resolution controls, additional primitives.
6. **Additional Features** – Import/use handling, 2D-only workflows, advanced modifiers, persisted projects.

## Contributing

1. Fork and clone the repository.
2. Keep changes scoped to one crate/layer at a time.
3. Add tests + documentation updates alongside code changes.
4. Run `cargo fmt && cargo clippy` plus relevant tests before opening a PR.
5. Describe architectural impacts in `specs/pipeline/plan-detailed.md` when introducing new patterns.

---
For architectural details and pending tasks, review `specs/pipeline/plan-detailed.md` and `specs/pipeline/tasks.md`.
