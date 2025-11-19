# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Rust Workspace Commands
```bash
# Build all workspace members
cargo build

# Build with release optimizations
cargo build --release

# Run all tests across workspace
cargo test

# Run tests for specific crate
cargo test -p manifold-rs
cargo test -p openscad-wasm

# Run specific test
cargo test -p manifold-rs cube_primitive

# Format code
cargo fmt

# Lint with clippy
cargo clippy

# Check without building
cargo check
```

### WASM Build Commands
```bash
# Build WASM package for playground (from workspace root)
node build-wasm.js

# Alternative: Use playground's build script
cd playground && pnpm run build:wasm
```

### Playground Development Commands
```bash
cd playground

# Start development server
pnpm dev

# Build playground
pnpm build

# Run playground tests
pnpm test

# Type checking
pnpm check

# Format code
pnpm format

# Lint code
pnpm lint
```

## Architecture

### Pipeline Overview
This is a Rust OpenSCAD-to-3D-Mesh pipeline targeting WebAssembly for browser-based playground use. The architecture follows strict vertical slices and single-responsibility principles:

```
OpenSCAD source text
        │
        ▼
libs/openscad-parser   (Tree-sitter CST via Rust bindings)
        │
        ▼
libs/openscad-ast      (Typed AST with spans)
        │
        ▼
libs/openscad-eval     (Evaluated AST / Geometry IR)
        │
        ▼
libs/manifold-rs       (Manifold Geometry → Mesh)
        │
        ▼
libs/wasm              (Interface-only orchestration)
        │
        ▼
playground             (Svelte + Three.js + Web Worker)
```

### Core Principles

1. **Vertical Slices**: Always implement one feature through the entire pipeline end-to-end
2. **Single Responsibility**: Each module has one clear purpose with its own `mod.rs` and `tests.rs`
3. **TDD First**: Write tests before implementation, following Red→Green→Refactor
4. **No Hidden Fallbacks**: All failures return explicit `Result<T, Error>` with diagnostics
5. **Zero-Copy Data Transfer**: Use `Float32Array` over WASM memory, never JSON for mesh data

### Crate Responsibilities

#### `libs/openscad-parser`
- Contains local Tree-sitter grammar (`grammar.js`) for OpenSCAD
- Generates Rust bindings in `bindings/rust/`
- Produces CST with source spans
- Parsing is encapsulated - Playground never imports Tree-sitter directly

#### `libs/openscad-ast`
- Owns typed AST data structures
- Converts CST to AST using parser bindings
- Exposes `build_ast_from_source()` and `build_ast()` functions
- CST parsing implementation is internal

#### `libs/openscad-eval`
- Interprets AST and produces Geometry IR
- Handles variables, modules, loops, functions
- Manages `EvaluationContext` with `$fn`, `$fa`, `$fs` resolution variables
- Pure data transformation: AST → Evaluated AST

#### `libs/manifold-rs`
- **Core geometry kernel** using direct port of C++ Manifold algorithms
- Index-based half-edge mesh representation with `Vec` arenas + `u32` indices
- All geometry calculations use `f64` precision (`glam::DVec3`)
- Implements primitives, transformations, and boolean operations
- **Sole owner** of mesh generation and export functionality
- Uses `rayon` for parallel processing

#### `libs/wasm`
- **Thin orchestration layer only** - no mesh or parsing logic
- Exposes high-level `compile_and_render(source: &str)` API
- Serializes rich diagnostics and handles panic hooks
- Bridges JavaScript to `manifold-rs` public APIs

### Critical Architecture Rules

1. **Module Boundaries**:
   - `openscad-parser` → `openscad-ast` → `openscad-eval` → `manifold-rs` → `wasm`
   - No skipping layers or direct dependencies between non-adjacent crates
   - `wasm` orchestrates via `manifold-rs` public APIs only

2. **Zero External Geometry Dependencies**:
   - `manifold-rs` is a direct C++ Manifold port, not a wrapper around `manifold3d` or `csgrs`
   - External libraries may be referenced but not used as runtime geometry kernels

3. **File Size Limits**: Keep files under 500 lines; split into SRP modules as needed

4. **Source Mapping**: Spans must be preserved throughout the pipeline for error diagnostics and geometry selection

### Data Structures

#### Geometry Precision
- Internal: `f64` (`glam::DVec3`, `DMat4`, `DQuat`)
- Export: `f32` only for GPU-bound data at kernel boundary
- `Vec3` type alias in `manifold-rs` points to `DVec3`

#### Index-Based Half-Edge Design
```rust
pub struct HalfEdgeMesh {
    vertices: Vec<Vertex>,
    half_edges: Vec<HalfEdge>,
    faces: Vec<Face>,
}
// All references use u32 indices, not pointers
```

#### Diagnostic Structure
```rust
struct Diagnostic {
    severity: Severity,  // Error, Warning
    message: String,
    span: Span,         // Source location
    hint: Option<String>,
}
```

### Development Workflow

1. **Vertical Slice Implementation**: Always implement one feature end-to-end through the pipeline
2. **Test-Driven Development**: Write tests first in `tests.rs` alongside implementation
3. **Error Handling**: All fallible operations return `Result<T, Error>` with explicit diagnostics
4. **Configuration**: All constants and magic numbers in `config.rs` per crate

### Performance Targets
- End-to-end compilation: <100ms for 10k triangle models
- Memory usage: <50MB for typical interactive models
- WASM bundle: <2MB compressed after optimization

### WASM Build System
The `build-wasm.js` script handles WASM compilation with:
- Zig toolchain integration for C++ dependencies
- `wasm-bindgen` for JavaScript bindings
- `wasm-opt` for bundle size optimization
- Environment variables for WASI sysroot configuration

### Testing Strategy
- Unit tests: Each module has `tests.rs` with comprehensive coverage
- Integration tests: Cross-crate pipeline testing
- Visual regression: Golden `.scad` → mesh hash tests
- Fuzz testing: Property-based tests for boolean operations using `proptest`
- Browser testing: `wasm-pack test --headless --chrome`

This architecture enables real-time OpenSCAD compilation entirely in the browser while maintaining the robustness needed for geometric operations.