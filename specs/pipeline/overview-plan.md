# Rust OpenSCAD Pipeline – Overview Plan

_Last updated: 2025-11-18 — incorporates 2025 Rust/WASM best practices._

> This document is the high-level source of truth for the Rust OpenSCAD pipeline. It describes **goals**, **architecture**, and **standards**. See `tasks.md` in the same folder for the detailed, phase-by-phase backlog.

---

## 1. Project Goal

Create a complete, robust, and performant **OpenSCAD-to-3D-Mesh pipeline** in Rust, targeting WebAssembly for a browser-based playground.

The system must:

- **Support real-time compilation** for interactive editing.
- **Run fully in the browser** via WASM.
- **Avoid unnecessary copies** between WASM and JS (zero-copy mesh transfer).
- **Provide precise source mapping** from errors and geometry back to OpenSCAD source.

All development must be broken down into **small, test-driven steps** that a developer can execute without needing external resources.

---

## 2. Core Philosophy (Strict Adherence Required)

- **Vertical Slices**  
  Implement one feature at a time through the *entire* pipeline:
  
  `Playground UI -> Worker -> WASM -> Parser -> AST -> Evaluator -> Manifold -> Mesh -> UI`

- **SRP & Structure**  
  Every *single-responsibility unit* (feature/struct/module) must live in its own folder with:
  
  - `mod.rs` – implementation
  - `tests.rs` – unit tests (TDD)
  
  Example:  
  `libs/manifold-rs/src/primitives/cube/{mod.rs, tests.rs}`.

- **TDD (Test-Driven Development)**  
  - Write tests **before** implementation.  
  - Workflow: **Red → Green → Refactor**.  
  - Prefer many small tests over large, fragile ones.

- **No Mocks (Except I/O)**  
  - Use real implementations in tests.  
  - Only mock external systems (e.g. filesystem, network).  
  - Avoid mocking internal logic.

- **Explicit Errors, No Fallbacks**  
  - Use `Result<T, Error>` everywhere for fallible operations.  
  - **Never** silently ignore failures or add hidden fallbacks.  
  - All failures must surface as explicit errors with diagnostics.

- **Centralized Configuration**  
  - All magic numbers and configuration must live in `config.rs` per crate.  
  - Complex or cross-crate configuration should be funneled through a small number of well-documented config modules.

- **Zero-Copy Data Transfer**  
  - Never serialize mesh data to JSON for rendering.  
  - Use `Float32Array` (and related typed arrays) over WASM memory.

- **File Size Limits**  
  - Max **500 lines per file**.  
  - If a file grows beyond this, split it into smaller SRP-driven modules.

- **Documentation & Examples**  
  - Every public function, struct, and module must have comments.  
  - Where practical, include small examples in comments or tests showing expected usage.

- **Readability Over Cleverness**  
  - Prefer code that is easy to read, debug, and extend over clever one-liners.  
  - Choose predictable control flow and clear naming.

- **DRY & KISS**  
  - Avoid duplication; refactor into shared utilities when patterns appear.  
  - Keep algorithms as simple as possible while remaining correct and performant.

- **Naming Matters**  
  - Names must be self-explanatory and describe intent.  
  - Avoid abbreviations unless they are absolutely standard (`pos`, `idx`, `fn`).

- **Continuous Cleanup**  
  - Remove legacy/unused code as you go.  
  - Keep specifications and documentation aligned with the implementation; delete outdated docs.

---

## 3. Architecture & Data Flow

High-level pipeline:

1. **Input**  
   OpenSCAD source text from the Playground editor.

2. **Parser – `libs/openscad-parser`**  
   - Tree-sitter grammar for OpenSCAD via bindings.  
   - Produces a typed **CST** with spans.

3. **AST – `libs/openscad-ast`**  
   - Converts CST into a typed **AST**.  
   - Every node carries a `Span { start: usize, end: usize }` for source mapping.

4. **Evaluator – `libs/openscad-eval`**  
   - Walks the AST, manages scopes, modules, functions, special vars (`$fn`, `$fa`, `$fs`).  
   - Produces **Geometry IR** (e.g. `GeometryNode::Cube { size, center, span }`).  
   - Includes a **memoization/caching layer**:
     - Key: `hash(AST node + scope variables)`.  
     - If a subtree and its dependencies are unchanged, reuse cached result.
   - Includes **recursion depth checks** and uses `stacker` to avoid WASM stack overflows for complex recursive scripts.

5. **Geometry Kernel – `libs/manifold-rs`**  
   - Consumes Geometry IR and outputs a manifold mesh.
   - Uses an **index-based Half-Edge** structure (Vec arenas + `u32` indices).  
   - Directly ports the algorithms from the local C++ Manifold library under `manifold/`, translating **algorithms**, not syntax.
   - Parallelism via **`rayon`**, replacing C++ `thrust` / TBB-style approaches.
   - Exposes a **safe, high-level Rust API** (e.g. `fn union(&self, other: &Self) -> Result<Self, Error>`), hiding raw half-edge details.
   - Ports **only** the features needed for the OpenSCAD vertical slices; new OpenSCAD features missing in C++ are implemented directly in Rust.

6. **Mesh Export (Kernel)**  
   - Performed exclusively in `libs/manifold-rs`.  
   - Converts internal `Manifold` representation into mesh buffers suitable for zero-copy WASM interfaces (e.g. `GlMeshBuffers`).  
   - Internal math uses `f64`; export to GPU-friendly `f32` only at the kernel boundary.

7. **WASM – `libs/wasm`**  
     - Thin interface-only orchestration between crates.  
     - Exposes kernel functionality from `libs/manifold-rs` (e.g. `compile_and_render(source: &str)`); no mesh logic or handlers in WASM.  
     - Serializes rich diagnostics from kernel error chains.  
     - Initializes panic hooks in debug builds.

8. **Playground**  
   - Svelte + Three.js front-end.  
   - Uses a Web Worker to invoke the WASM pipeline.  
   - Receives typed arrays and constructs `THREE.BufferGeometry` without copying.

### 3.1 Precision & Robustness

- **Floating Point Precision**  
  - All geometry calculations use `f64`.  
  - Types: `glam::DVec3`, `DMat4`, `DQuat`.  
  - `Vec3` type alias in `libs/manifold-rs` must point to `DVec3`.

- **Export Precision**  
  - `f32` is allowed **only** for GPU-bound data in `GlMeshBuffers`.

- **2D Operations (Clipper2)**  
  - When converting `f64` → `i64`, use a **standardized scaling factor** (e.g. `1e6`) configured centrally in `config.rs` (for example a `CLIPPER_SCALE` constant).  
  - Avoid ad-hoc scaling to prevent grid-snapping artifacts.

- **Robust Predicates**  
  - Use the `robust` crate for exact predicates (e.g. orientation, incircle).  
  - Do **not** rely on naive epsilon comparisons for validity checks.

### 3.2 Source Mapping & Diagnostics

- Spans (`Span { start, end }`) must be preserved from:  
  `Parser -> AST -> Evaluator -> Geometry IR -> Mesh/diagnostics`.

- A common `Diagnostic` struct is used across the pipeline:

  ```rust
  struct Diagnostic {
      severity: Severity,  // Error, Warning
      message: String,
      span: Span,
      hint: Option<String>,
  }
  ```

- The Playground uses `Diagnostic` data for:
  - Editor squiggles.  
  - Mapping 3D triangle selections back to source.

### 3.3 Crate Dependency Graph

The workspace must form a clear, acyclic dependency graph:

```text
apps/playground (SvelteKit + Three.js)
  └─> wasm
        └─> manifold-rs        (consumes Evaluated AST, generates Mesh)
              └─> openscad-eval   (produces Evaluated AST / IR)
              └─> openscad-ast    (builds AST from CST)
                          └─> openscad-parser   (produces CST from source)

editors (VSCode, Neovim, etc.)
  └─> openscad-lsp          (tower-lsp + Tree-sitter)
        └─> openscad-ast?   (optional, for semantic features)
                └─> openscad-parser
```

Key rules:

- `openscad-parser` takes OpenSCAD source and produces a Tree-sitter CST.
- `openscad-ast` depends on `openscad-parser` to build typed AST nodes.
- `openscad-eval` consumes `openscad-ast` and produces an **Evaluated AST** (Geometry IR). It does **not** depend on `manifold-rs`.
      - `manifold-rs` consumes the Evaluated AST from `openscad-eval` and generates the geometry.
      - `wasm` orchestrates via `manifold_rs` public APIs only (e.g. `compile`, `process_openscad`), and does not implement mesh logic or import parser/AST crates directly.

### 3.4 Library Responsibilities & Relationships

- **`libs/openscad-parser`**  
  - Boundary to Tree-sitter, implemented entirely in Rust.  
  - Uses the generated Tree-sitter parser and its Rust bindings to turn **OpenSCAD source text** into a **CST** (Concrete Syntax Tree).

- **`libs/openscad-ast`**  
  - Owns the **typed AST** data structures.  
  - Converts CST to AST.

- **`libs/openscad-eval`**  
  - Interprets the AST and produces **Evaluated AST** (Geometry IR).  
  - Handles variables, modules, loops, functions.  
  - **Pure data transformation**: AST -> Evaluated AST. No geometry generation.

- **`libs/manifold-rs`**  
  - Geometry kernel and manifold mesh implementation.  
  - **Consumes Evaluated AST** to construct the Manifold geometry.  
  - Exposes high-level APIs to convert OpenSCAD source (via eval) into Meshes.

- **`libs/wasm`**  
  - Thin bridge.  
  - Calls `manifold-rs` to process code and return mesh buffers.  
  - No mesh logic, no parsing logic.  
  - Produces the **only browser-facing WASM bundle**: the wasm-bindgen output under `libs/wasm/pkg` (for example `wasm.js` + `wasm_bg.wasm`), analogous to Tree-sitter's `web-tree-sitter.{js,wasm}` pair.  
  - Is consumed by TypeScript via a small wrapper (for example `initWasm`, `compile(...)`) that mirrors `web-tree-sitter`'s `Parser.init()` + `parser.parse()` pattern.

- **`libs/openscad-lsp`**  
  - Rust-native Language Server built with `tower-lsp`.  
  - Uses `libs/openscad-parser` (Tree-sitter grammar + Rust bindings) to maintain incremental parse trees for open documents.  
  - Provides editor features such as diagnostics, document symbols, and later go-to-definition/completion via high-level domain types.  
  - Runs outside WASM (as a native process) and never exposes raw `tree_sitter::Node` values to clients.

#### 3.4.1 Data Flow Diagram

```text
OpenSCAD source text
        │
        ▼
libs/openscad-parser   (CST)
        │
        ▼
libs/openscad-ast      (AST)
        │
        ▼
libs/openscad-eval     (Evaluated AST / IR)
        │
        ▼
libs/manifold-rs       (Manifold Geometry -> Mesh)
        │
        ▼
libs/wasm              (MeshHandle)
        │
        ▼
apps/playground
```

### 3.5 Minimal `cube(10);` Pipeline (Simplified)

For the initial tracer-bullet implementation, all layers must participate in a minimal `cube(10);` flow with no shortcuts:

- **Playground**  
  - The user edits `cube(10);` in the OpenSCAD editor and triggers a compile/render action.
- **`libs/wasm`**  
  - Receives the source string and exposes a single entry point (for example `compile_and_render` or `compile_and_count_nodes`).  
  - Forwards the raw OpenSCAD source to `libs/manifold-rs` as the orchestrator of the geometry pipeline.
- **`libs/manifold-rs`**  
  - Calls into `libs/openscad-eval` with the original OpenSCAD source string `cube(10);` to obtain an **evaluated/flattened AST**.  
  - Transforms this evaluated AST into a mesh and returns it to `libs/wasm`.
- **`libs/openscad-eval`**  
  - Invokes `libs/openscad-ast` with the original source so that the AST layer can obtain a typed AST from the CST.  
  - Decides whether evaluation is required for the current AST, evaluates and resolves it, and returns an evaluated/flattened AST to `libs/manifold-rs`.
- **`libs/openscad-ast`**  
  - Calls `libs/openscad-parser` with `cube(10);` to obtain a Tree-sitter CST.  
  - Transforms the CST into a typed AST and returns it to `libs/openscad-eval`.
- **`libs/openscad-parser`**  
  - Parses the OpenSCAD source into a CST using the Tree-sitter grammar and returns it to `libs/openscad-ast`.
- **Back up to geometry and rendering**  
  - `libs/manifold-rs` converts the evaluated AST for `cube(10);` into a mesh and returns this mesh to `libs/wasm`.  
  - `libs/wasm` returns the generated mesh to the Playground as typed buffers/handles.  
  - The Playground converts the mesh buffers into Three.js geometry and renders the cube.

This minimal vertical slice is the reference "happy path" for all future primitives and features.

---

## 4. Implementation Phases (High-Level)

A vertical slice is always preferred over broad, unfinished scaffolding.

- **Phase 0 – Pre-Bootstrap Evaluation**  
  - Confirm the direct-port strategy for `libs/manifold-rs` (from the local C++ Manifold library using an index-based half-edge) and confirm `libs/openscad-parser/src/grammar.json` as the canonical Tree-sitter grammar before committing to a long-term architecture.

**Phase 0 Confirmation (Task 0.2):**

`libs/openscad-parser/src/grammar.json` confirmed as canonical:

- Coverage: primitives (`module_call` e.g. `cube`), transforms (`transform_chain` e.g. `translate`), booleans (`union_block`, `module_call` e.g. `difference`), control flow (`for_block`/`if_block`), advanced (`module_item`/`function_item`/`include_statement`).

- Build: `tree-sitter generate` successful (src/parser.c compiled via `bindings/rust/build.rs`).

- Bindings: `bindings/rust/lib.rs` provides `language()`; `test_can_load_grammar()` validates loading.

No further setup required for Phase 1 integration.

- **Phase 1 – Infrastructure & Tracer Bullet**  
  - Set up the Cargo workspace, core crates, `libs/openscad-lsp`, and basic Svelte+WASM Playground.  
  - Implement minimal “hello world” paths:  
    - Playground → Worker → WASM (for example, echo a string or return a trivial mesh).  
    - Editor → `openscad-lsp` (a tower-lsp server that can parse a file and publish basic syntax diagnostics).
  - *Current status:* `apps/playground` already contains a minimal SvelteKit setup; Three.js scene, worker wiring, and WASM/`manifold-rs` geometry preview are still pending.

- **Phase 2 – First Primitive (Cube)**  
  - Implement a fully working `cube()` primitive end-to-end.  
  - Include AST, IR, Evaluator, `manifold-rs` cube, and zero-copy rendering.  
  - Introduce diagnostics and basic error handling.

- **Phase 3 – Filesystem & Parameters**  
  - Add named arguments and a `FileSystem` abstraction with a virtual in-memory implementation for WASM.  
  - Introduce an `EvaluationContext` that tracks resolution special variables (`$fn`, `$fa`, `$fs`) and other global parameters.  
  - Support `include` / `use` semantics.

- **Phase 4 – Sphere & Resolution Controls**  
  - Implement `sphere()` via icosphere subdivision.  
  - Use the `EvaluationContext` resolution parameters when tessellating the sphere.

- **Phase 5 – Transformations**  
  - Support `translate`, `rotate`, `scale` transformations in IR and `manifold-rs`.  
  - Ensure transformations preserve and update Spans for diagnostics.

- **Phase 6 – Boolean Operations**  
  - Implement robust `union`, `difference`, and `intersection`.  
  - Use robust predicates, R-Tree or BVH for broad phase, and correct retriangulation.  
  - Add fuzz testing to validate manifold invariants.

A detailed breakdown of tasks, subtasks, and acceptance criteria for each phase lives in `tasks.md`.

---

## 5. Coding Standards & Global Guidelines

### 5.1 Rust & Crate-Level Standards

- **SRP & Folder Layout**  
  - Each SRP unit resides in its own folder with `mod.rs` and `tests.rs`.  
  - Example: `libs/manifold-rs/src/primitives/cube/{mod.rs, tests.rs}`.

- **File Length**  
  - Maximum 500 lines per file; refactor and split as needed.

- **Error Handling Policy**  
  - No silent fallbacks or best-effort behaviour on failure.  
  - Return explicit errors, with diagnostics and spans, for all failure modes.

- **Configuration**  
  - All constants and tunable parameters go into `config.rs`.  
  - Cross-cutting precision parameters (like `EPSILON`) can be toggled via Cargo features (e.g. `high_precision`).

- **Testing & TDD**  
  - Write tests first for every new behaviour.  
  - Use unit tests for small units and integration tests for cross-crate flows.  
  - Add **visual regression tests** (golden `.scad` → `.stl`/`.obj` hashes).  
  - Add **fuzz tests** for boolean operations using `proptest`.

- **Performance & Safety**  
  - Prefer safe Rust; use `unsafe` only inside small, well-audited half-edge kernels encapsulated behind safe APIs.  
  - Enable overflow checks in builds that guard geometry math (e.g. `overflow-checks = true` in appropriate profiles), especially in debug and fuzzing configurations.  
  - Use `rayon` for safe data-parallel operations when beneficial.

### 5.2 WASM & TypeScript Standards

- **TypeScript Types**  
  - No `any` types. Always use precise TypeScript types or generics.  
  - Model WASM handles and messages with clear interfaces.

- **WASM Memory Lifecycle**  
  - Pattern: **Alloc → View → Upload → Free**.
  - Rust allocates mesh buffers and returns a handle.  
  - JS creates a typed view (`Float32Array`/`Uint32Array`) over WASM memory and uploads to GPU.  
  - JS **must** call a Rust-exported `free_*` function (or invoke `.free()` on a wrapper) immediately after upload.

- **Panic Debugging**  
  - `console_error_panic_hook` must be active for debug builds to avoid opaque `unreachable` panics.

- **Playground Architecture**  
  - Heavy computation must occur in a Web Worker.  
  - Main thread only handles UI and rendering.

- **Playground Tooling Stack**  
  - `apps/playground` uses Svelte 5 with SvelteKit, Vite 7, Vitest 4, TypeScript 5.9, ESLint 9, and plain `three` (no Svelte wrapper library).  
  - All `pnpm` scripts (dev, test, lint) must complete with zero TypeScript and ESLint errors.

- **Async & Long-Running Operations**  
  - Expose long-running operations (e.g. boolean CSG) as async WASM functions (via `wasm-bindgen-futures`) and `await` them in the worker to avoid blocking its event loop.

- **TypeScript Wrapper for WASM**  
  - All raw pointer logic lives in a small, well-tested TS wrapper module.  
  - The wrapper returns higher-level objects (e.g. `Mesh`) with explicit `dispose()`/`free()` methods and/or `FinalizationRegistry` for safety.

- **WASM Runtime Bundle**  
  - The only browser-facing WASM entrypoint is the wasm-bindgen glue in `libs/wasm/pkg/wasm.js` and its `wasm_bg.wasm` binary, consumed via the `$wasm` alias in `apps/playground`.  
  - Build tooling such as `scripts/build-wasm.sh` and any Node-based helper (for example `build-wasm.js`) is strictly CLI-only and must never be imported into browser bundles, mirroring Tree-sitter's separation between its `binding_web` runtime and `script/build.js`.

- **WASM Parallelism**  
  - Where browser support allows, enable WASM threads + shared memory so `rayon` can run in parallel inside `manifold-rs` for heavy kernels.

- **Local WASM Build & Distribution**  
  - `libs/wasm` is built for the `wasm32-unknown-unknown` target via dedicated helpers (`scripts/build-wasm.sh` on Unix-like systems and a Node CLI equivalent on Windows, such as `build-wasm.js`).  
  - These helpers ensure the `wasm32-unknown-unknown` target and a compatible `wasm-bindgen` CLI are installed, respect `WASI_SDK_PATH` when compiling any C/C++ dependencies, run `cargo build --release -p wasm --target wasm32-unknown-unknown`, and invoke `wasm-bindgen --target web` to emit artifacts into `libs/wasm/pkg`.  
  - `apps/playground` pnpm scripts (for example `pnpm build:wasm`) call these helpers so that the `libs/wasm/pkg/wasm.js` + `wasm_bg.wasm` bundle used in the browser is always up to date.

### 5.3 Documentation & Project Hygiene

- Keep `overview-plan.md` and `tasks.md` up to date with implementation changes.  
- Remove obsolete sections instead of letting them drift.  
- Ensure examples in comments and tests remain correct and relevant.

### 5.4 Performance Targets

- For typical interactive models (~10k triangles), aim for end-to-end compile times under ~100 ms on a mid-tier 2025 laptop browser, and **≤ 50 ms** for boolean-heavy ~10k triangle scenes when WASM threads + `rayon` are available.  
- Use tools such as `wasm-opt -O4`, dead-code elimination, and profiling to keep the WASM bundle lean.  
- Benchmark with synthetic scenes generated via property/fuzz tests to capture worst-case CSG patterns.

---

## 6. References

These internal resources are the primary references for the implementation:

- **OpenSCAD Syntax: Grammar Definition and Test Coverage**  
  - `libs/openscad-parser/src/grammar.json` (canonical Tree-sitter grammar)  
  - `libs/openscad-parser/test/corpus/**`  
  - Optionally, an upstream OpenSCAD grammar such as `holistic-stack/tree-sitter-openscad` may be consulted for reference or additional test cases, but `grammar.json` remains the source of truth.

- **Manifold Geometry Kernel: C++ Implementation & Porting Strategy**  
  - C++ Manifold sources under `manifold/` (e.g. `impl.h`, `impl.cpp`, `boolean3.*`, `constructors.cpp`).  
  - Rust port lives in `libs/manifold-rs/` with an index-based half-edge mesh and safe public API.  
  - External crates such as `manifold3d` or `csgrs` may be consulted as references or benchmarks, but **must not** be used as runtime geometry kernels behind `libs/manifold-rs`.

All new work should keep this overview in sync, and `tasks.md` should always reflect the current state of the actionable backlog.

---

## 7. CI/CD & Testing Pipeline

- Use GitHub Actions (or an equivalent CI system) to run on every PR:
  - `cargo fmt`, `cargo clippy`, and `cargo test` for all crates.  
  - WASM tests (e.g. `wasm-pack test --headless --chrome` for `libs/wasm` or related crates) run **inside the same Dockerized Rust+WASM image** used for builds, not directly on the CI host.  
  - Playwright end-to-end tests for the Playground.  
  - Golden regression tests and periodic fuzz tests (at least nightly).
- Treat failing CI as a blocker; update this plan and `tasks.md` alongside behavioural changes.
