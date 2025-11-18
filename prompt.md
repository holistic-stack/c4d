Create Rust OpenSCAD Pipeline: Comprehensive Master Plan in overview-plan.md, tasks.md files in specs/pipeline/ folder;

## 1. Project Goal
Create a complete, robust, and performant **OpenSCAD-to-3D-Mesh pipeline** in Rust, targeting WebAssembly for a browser-based playground. The system must support real-time compilation, zero-copy data transfer, and source mapping.
breakdown the plan in to max smaller steps/tasks/subtasks the steps with all necessary information for the developer needs without need external resources;

## 2. Core Philosophy (Strict Adherence Required)
*   **Vertical Slices**: Implement one feature at a time through the *entire* pipeline (UI -> WASM -> Parser -> AST -> Evaluator -> Manifold -> Mesh -> UI).
*   **SRP & Structure**: Every "SRP unit" (feature/struct) must have its own folder with `mod.rs` and `tests.rs`.
    *   Example: `libs/manifold-rs/src/primitives/cube/{mod.rs, tests.rs}`.
*   **TDD**: Write `tests.rs` **before** implementation. Red -> Green -> Refactor.
*   **No Mocks**: Use real implementations for internal logic. Mock only I/O boundaries (like FileSystem).
*   **Explicit Errors**: Use `Result<T, Error>`. No panics, no silent fallbacks.
*   **Centralized Config**: All magic numbers/defaults in `config.rs` per crate.
*   **Zero-Copy**: Never serialize mesh data to JSON. Use `Float32Array` views into WASM memory.
*   **File Limits**: Max 500 lines per file. Split if larger.

## 3. Architecture & Data Flow
1.  **Input**: OpenSCAD string from Playground Editor.
2.  **Parser (`libs/openscad-parser`)**: Tree-sitter generates CST.
3.  **AST (`libs/openscad-ast`)**: Typed AST with `Span` (source mapping).
4.  **Evaluator (`libs/openscad-eval`)**: Walks AST, handles Scope/Modules, produces **Geometry IR**. Includes a **Memoization/Caching Layer** (Key = Hash(AST Node + Scope Variables)).
5.  **Geometry Kernel (`libs/manifold-rs`)**:
    *   **Role**: Consumes IR, uses **Index-Based Half-Edge** structure (Vec arenas + u32 indices), produces `Manifold`.
    *   **Source**: **MUST** be ported from the local C++ `manifold/` repository.
    *   **Strategy**: Port **algorithms** (Logic), not syntax. Use `rayon` for parallelism (instead of `thrust`). Avoid C++ inheritance; use Rust Traits/Enums. Prioritize readability.
    *   **Scope**: Port **ONLY** features required for OpenSCAD (Vertical Slice). If OpenSCAD needs a feature that Manifold C++ lacks, implement it in Rust.
6.  **Mesh Export**: Converts to `MeshGL` (flat f32 buffers).
7.  **WASM (`libs/wasm`)**: Exposes `MeshHandle` (pointers) to JS.
8.  **Playground**: Web Worker executes pipeline, Three.js renders via Zero-Copy.

## 4. Implementation Plan (Detailed Steps)

### Phase 1: Infrastructure & The "Tracer Bullet"

#### Task 1.1: Workspace & Crate Setup
*   **Goal**: Initialize Cargo workspace and core crates.
*   **Action**:
    *   `Cargo.toml` [workspace] members: `libs/openscad-parser`, `libs/openscad-ast`, `libs/openscad-eval`, `libs/manifold-rs`, `libs/wasm`.
    *   Create `libs/manifold-rs`: Deps: `glam` (f64), `thiserror`, `robust`.
        *   Structure: `src/core/vec3/mod.rs` (Type alias `pub type Vec3 = glam::DVec3;`), `src/{lib.rs, config.rs}`.
    *   Create `libs/openscad-eval`: Deps: `glam`, `stacker`.
        *   **Recursion**: Implement explicit recursion depth checks + `stacker` to prevent WASM Stack Overflow.
    *   Create `libs/wasm`: Type: `cdylib`. Deps: `wasm-bindgen`, `console_error_panic_hook`.
        *   **Init**: `lib.rs` must initialize `console_error_panic_hook` for debuggable panics.
    *   **Config**: `libs/manifold-rs/src/config.rs` with `EPSILON`, `DEFAULT_SEGMENTS`.

#### Task 1.2: Playground Setup (Svelte + Three.js + Worker)
*   **Goal**: Render engine with Web Worker support.
*   **Action**:
    *   Setup SvelteKit/Vite project in `playground/`.
    *   **Worker**: `src/worker/pipeline.worker.ts`. Handles WASM loading and execution.
    *   **Glue Code**: Create a TypeScript Wrapper to encapsulate unsafe pointers and ensure memory safety.
        *   Pattern: `const mesh = wasm.compile(src);` (Returns object handling `free()` via `FinalizationRegistry` or explicit `dispose()`).
        *   Requirement: The TypeScript Wrapper must handle memory management and ensure that the WASM memory is properly cleaned up after use.
    *   **Scene**: `src/components/viewer/scene-manager.ts` (SRP). Three.js setup (Lights, Camera, OrbitControls).
    *   **Zero-Copy Logic**:
        1. Worker calls WASM -> gets `MeshHandle`.
        2. Worker creates `new Float32Array(wasm.memory.buffer, ptr, len)`.
        3. Worker `postMessage` with `transfer` [buffer].
        4. Main thread receives buffer -> `THREE.BufferGeometry`.

#### Task 1.3: The "Hello World"
*   **Goal**: Verify WASM connectivity.
*   **Action**: `libs/wasm` exports `greet()`. Playground calls it.

#### Task 1.4: Parser Infrastructure (WASM)
*   **Goal**: Tree-sitter running in Browser.
*   **Action**:
    *   `libs/openscad-parser`: Build script for C grammar.
    *   **Playground**: `npm install web-tree-sitter`.
    *   **Assets**: Copy `tree-sitter.wasm` and `tree-sitter-openscad.wasm` to `public/`.
    *   **Init**: `src/worker/parser-init.ts` initializes parser.

---

### Phase 2: The First Primitive (Cube)

#### Task 2.1: Manifold-RS Cube (TDD)
*   **Location**: `libs/manifold-rs/src/primitives/cube/`.
*   **Tests**: `tests.rs` -> Assert 8 vertices, 12 triangles.
*   **Impl**: `mod.rs` -> `pub fn cube(size: DVec3, center: bool) -> Manifold`.
*   **Validation**: Implement `Manifold::validate()` (Euler check).

#### Task 2.2: AST & Source Mapping
*   **Location**: `libs/openscad-ast/src/nodes/`.
*   **Struct**: `pub struct Span { start: usize, end: usize }`.
*   **Update**: Add `span: Span` to all AST nodes.
*   **Parser**: Extract `start_byte`/`end_byte` from tree-sitter and populate Span.

#### Task 2.3: Geometry IR & Evaluator
*   **Location**: `libs/openscad-eval/src/ir/`.
*   **Enum**: `GeometryNode::Cube { size: DVec3, center: bool, span: Span }`.
*   **Evaluator**: `libs/openscad-eval/src/evaluator/`.
    *   `eval_call` maps `cube()` to `GeometryNode::Cube`.

#### Task 2.4: Pipeline Integration & Error Reporting
*   **Diagnostic**: Define standard struct:
    ```rust
    struct Diagnostic { severity: Severity, message: String, span: Span, hint: Option<String> }
    ```
*   **WASM**: `compile_and_render(source) -> Result<MeshHandle, Vec<Diagnostic>>`.
*   **MeshHandle**: Expose `primitive_ids_ptr` (maps triangles to Spans).
*   **Playground**:
    *   **Diagnostics**: Show "Squiggly Lines" for errors (using Span).
    *   **Picking**: Click 3D Mesh -> Trace `primitive_id` -> Highlight Code.

#### Task 2.5: Visual Regression & Golden Files (New)
*   **Goal**: Catch geometric regressions invisible to unit tests.
*   **Action**:
    *   Create `tests/golden/` directory with standard `.scad` files.
    *   Implement a test runner that compiles these to STL/OBJ.
    *   Compare output hashes against checked-in "Golden" hashes.

---

### Phase 3: Filesystem & Parameters

#### Task 3.1: Named Arguments
*   **Evaluator**: Handle `cube(size=[10,10,10], center=true)`. Map args by name.

#### Task 3.3: FileSystem Abstraction
*   **Trait**: `libs/openscad-eval/src/filesystem.rs` -> `trait FileSystem { fn read_file(...) }`.
*   **WASM Impl**: `VirtualFileSystem` (HashMap in memory).
*   **Logic**: `eval(..., fs: &dyn FileSystem)`. Handle `include` / `use`.

---

### Phase 4: The Sphere & Resolution

#### Task 4.1: Sphere (Icosphere)
*   **Location**: `libs/manifold-rs/src/primitives/sphere/`.
*   **Algo**: Icosphere subdivision.
*   **Config**: `libs/manifold-rs/src/config.rs` -> `DEFAULT_FN = 32`.
*   **Context**: Evaluator manages `$fn`, `$fa`, `$fs` in `Context` struct.

---

### Phase 5: Transformations

#### Task 5.1: Transform Nodes
*   **IR**: `GeometryNode::Transform(Mat4, Box<GeometryNode>)`.
*   **Manifold**: Implement `translate`, `rotate`, `scale` (Matrix multiplication).

---

### Phase 6: Boolean Operations

#### Task 6.1: Robust Predicates
*   **Location**: `libs/manifold-rs/src/predicates/`.
*   **Dep**: `robust` crate. Replace `dot > EPSILON` with `orient3d`.

#### Task 6.2: Boolean Logic (CSG)
*   **Location**: `libs/manifold-rs/src/boolean/{union, difference, intersection}/`.
*   **Algo**:
    1. Broad phase (R-Tree).
    2. Exact phase (Retriangulation).
    3. Classification (Inside/Outside).
*   **Sanitation**: Remove degenerate triangles.

#### Task 6.3: Fuzz Testing
*   **Tool**: `proptest`.
*   **Logic**: Generate random primitives/transforms -> Union.
*   **Invariant**: `Manifold::validate()` must always return true. No panics allowed.

---

## 5. Coding Standards Summary
*   **Floating Point Precision**: Internal math **MUST** be `f64`. Export **ONLY** as `f32`.
*   **Wasm Panics**: `console_error_panic_hook` must be active in debug builds.
*   **WASM Memory**: Follow "Alloc -> View -> Upload -> Free". JS **MUST** call `handle.free()` immediately after GPU upload.
*   **Folder Structure**: `libs/crate/src/feature/{mod.rs, tests.rs}`.
*   **Naming**: Folders (`kebab-case`), Rust (`snake_case`), Types (`PascalCase`).
*   **Error Handling**: Use `thiserror`. No fallbacks.
*   **Testing**:
    *   Unit: `cargo test`.
    *   WASM: `wasm-pack test --headless`.
    *   Browser: Playwright (`npm run test:playwright`).

## 6. Immediate Next Steps (User Instructions)
To start working, verify the **current state** and pick the next **Task** from Phase 1 or 2. Always verify `Cargo.toml` workspace members first.


