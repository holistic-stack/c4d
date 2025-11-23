# Rust OpenSCAD Pipeline – Task Breakdown

> This file is the **actionable backlog** for the Rust OpenSCAD pipeline.  
> It is structured into small, test-driven tasks and subtasks.  
> See `overview-plan.md` for goals, architecture, and coding standards.

---

## Conventions

- Each task is **small**, **TDD-first**, and adheres to the **SRP**.
- No mocks, except for filesystem or other external I/O boundaries.
- All failures must return explicit errors; no hidden fallbacks.
- All geometry code uses `f64` internally; `f32` is only for GPU export.
- Keep files under **500 lines**; split as soon as they grow too large.
 - For every new public API (functions, structs, modules), add Rust doc-comments with at least one minimal usage example, and keep examples compiling as part of tests.

For each task, we list:

- **Goal** – What we want to achieve.  
- **Steps** – Concrete actions to perform.  
- **Acceptance Criteria** – How we know the task is done.

---

## Phase 0 – Pre-Bootstrap Evaluation

### Task 0.1 – Confirm Geometry Kernel Strategy (Direct Port)

**Goal**  
Confirm and document that `libs/manifold-rs` will be a direct port of the local C++ Manifold algorithms (index-based half-edge), not a thin wrapper around external kernels like `manifold3d` or `csgrs`.

**Steps**

1. Survey `manifold/` C++ sources to outline the main data structures and algorithms to port into `libs/manifold-rs`.  
2. Optionally review `manifold3d` Rust bindings and the `csgrs` crate **only as references** (for ideas, tests, or benchmarks), not as runtime geometry kernels.  
3. Write a short design note describing:
   - The chosen approach: direct port into `libs/manifold-rs` with an index-based half-edge.  
   - Which parts of C++ Manifold are in scope for the first vertical slices.  
   - An explicit statement that `manifold3d` and `csgrs` will not be used as runtime geometry kernels behind `libs/manifold-rs`.

**Acceptance Criteria**

- A design note exists in the repo (e.g. `specs/pipeline/kernel-decision.md`) clearly stating that `libs/manifold-rs` is a direct C++ Manifold port and that `manifold3d`/`csgrs` are not used as interim kernels.  
- `overview-plan.md` and the dependency graph remain valid for this approach.

### Task 0.2 – Confirm Tree-sitter Grammar Integration

**Goal**  
Confirm that `libs/openscad-parser` is the canonical Tree-sitter grammar for OpenSCAD and that it is correctly wired into the Rust parser crate.

**Steps**

1. Verify that `libs/openscad-parser/src/grammar.json` builds successfully with Tree-sitter and covers primitives, transforms, booleans, control flow, and advanced features required for the initial slices.  
2. Ensure the Rust bindings under `libs/openscad-parser/bindings/rust/lib.rs` use this `grammar.json` when generating the Tree-sitter parser.  
3. Document `grammar.json` as the canonical grammar file in `overview-plan.md` and this task file.

**Acceptance Criteria**

- `libs/openscad-parser/src/grammar.json` is clearly documented as the canonical grammar in `overview-plan.md` and this task file.  
## Phase 1 – Infrastructure & "Tracer Bullet"
**Task 0.2 Confirmation:**

- `grammar.json` verified: rules cover primitives (e.g. `cube()` via `module_call`), transforms (`translate`/`rotate`/`scale` via `transform_chain`), booleans (`union_block`, `difference`/`intersection` via `module_call`), control flow (`for_block`/`if_block`), advanced (`module_item`/`function_item`/`include_statement`/`use_statement`).

- Builds successfully: `src/parser.c` generated (implied by `bindings/rust/build.rs`), `tree_sitter/` setup present.

- Rust bindings confirmed: `bindings/rust/lib.rs` exports `language()` function loading the grammar; includes `test_can_load_grammar()`.

- No setup needed for Phase 1; integration ready.

- Files readable; grammar.json (~2279 formatted lines) is canonical JSON data.

### Task 1.1 – Workspace & Crate Setup 

**Goal**  
Initialize the Cargo workspace and core Rust crates from scratch, with proper dependencies and configuration.

**Steps**

1. **Workspace Configuration**  
   - Update root `Cargo.toml` to include these members:  
     - `libs/openscad-parser`  
     - `libs/openscad-ast`  
     - `libs/openscad-eval`  
     - `libs/manifold-rs`  
     - `libs/wasm`  
     - `libs/openscad-lsp`  

2. **Create `libs/manifold-rs`**  
   - Create crate structure:  
     - `src/lib.rs`  
     - `src/config.rs`  
     - `src/core/vec3/mod.rs` (type alias `pub type Vec3 = glam::DVec3;`).  
   - Add dependencies in `Cargo.toml`:  
     - `glam` (f64 support)  
     - `thiserror`  
     - `robust`  
     - `rayon`  
   - Define `config.rs` with core constants (e.g. `EPSILON`, `DEFAULT_SEGMENTS`) and document their purpose.

3. **Create `libs/openscad-eval`**  
   - Create structure:  
     - `src/lib.rs`  
     - `src/ir/mod.rs`  
     - `src/evaluator/mod.rs`  
     - `src/filesystem.rs`  
   - Add dependencies:  
     - `glam`  
     - `stacker`  
   - Add a basic `GeometryNode` enum scaffold under `ir/` (cube variant can initially be a stub, to be filled in Phase 2).  
   - Implement recursion depth guards and configure `stacker` for deeper recursion in evaluator entry points.

4. **Create `libs/wasm`**  
   - Crate type: `cdylib`.  
   - Dependencies:  
     - `wasm-bindgen`  
     - `console_error_panic_hook`  
   - In `src/lib.rs`:  
     - Initialize `console_error_panic_hook` in an `init` function called by JS in debug builds.  
     - Add a small placeholder export (e.g. `greet()`), to be replaced by real pipeline functions in later tasks.  
   - Ensure the crate compiles for the `wasm32-unknown-unknown` target **inside a Dockerized Rust+WASM image** (see Task 1.3), and, if using `rayon` on WASM, that appropriate thread/atomics features, bulk-memory, and linker flags are enabled for future WASM threads.

**Acceptance Criteria**

- `cargo build` at workspace root succeeds without errors.  
- `cargo test` (with initial no-op tests) succeeds for all new crates.  
- Crate dependency graph matches the architecture in `overview-plan.md` (no unexpected cycles).

**Task 1.1 Status:**

- Workspace members added for `config`, `libs/openscad-parser`, `libs/openscad-ast`, `libs/openscad-eval`, `libs/manifold-rs`, `libs/wasm`, and `libs/openscad-lsp`.  
- Core crates scaffolded with SRP modules (`mod.rs` + `tests.rs`) and documented public APIs referencing centralized `config` constants.  
- `libs/wasm` exposes a panic hook initializer and a minimal `compile_and_count_nodes` pipeline stub wired into `openscad-eval`.  
- `openscad-lsp` provides a stdio server binary, in-memory document store, and a minimal parser stub sufficient for end-to-end testing.  
- `cargo fmt` and `cargo test` at the workspace root both succeed.

---

### Task 1.2 – Playground Setup (Svelte + Three.js + Worker) 

**Goal**  
Set up `apps/playground` with a Web Worker and Three.js scene, ready to call the real `libs/wasm` bundle (no mocks).

**Steps**

1. **Project Initialization**  
   - Under `apps/playground/`, initialize a SvelteKit project.  
   - Use **pnpm** as the package manager (`pnpm dev`, `pnpm test`, `pnpm lint`).  
   - Use Svelte 5 with SvelteKit, Vite 7, Vitest 4, TypeScript 5.9, ESLint 9, and plain `three` (no Svelte wrapper library).  
   - Enable strict TypeScript mode.  
   - Enforce `kebab-case` for TypeScript file and folder names (e.g. `mesh-wrapper.ts`, `pipeline.worker.ts`).  
   - Add a `build:wasm` script to `package.json` that runs `../scripts/build-wasm.sh`, which drives the local Rust+wasm-bindgen toolchain.

2. **Web Worker for Pipeline**  
   - Create `src/worker/pipeline.worker.ts`.  
   - Responsibilities:  
     - Load the actual WASM bundle produced from `libs/wasm` (built via `../scripts/build-wasm.sh`), not a mock module.  
     - Expose a message protocol for `compile(source: string)`.  
     - Forward diagnostics and mesh handles back to the main thread.

3. **TypeScript Wrapper for WASM (Glue Code)**  
   - Create `src/lib/wasm/mesh-wrapper.ts` that:  
     - Encapsulates raw pointer handling (`ptr`, `len`) from `MeshHandle`.  
     - Provides a `Mesh` class or interface with typed views over WASM memory (`Float32Array`, `Uint32Array`).  
     - Exposes a `dispose()`/`free()` method that calls the Rust `free_*` entry point.  
     - Uses precise TypeScript types (no `any`), and defines explicit interfaces for `MeshHandle`, diagnostics, and worker messages.

4. **Three.js Scene Manager**  
   - Implement `src/components/viewer/scene-manager.ts` with SRP:  
     - Set up renderer, camera, lights, and controls.  
     - Expose functions to attach to a canvas and update geometry from provided buffers.

**Acceptance Criteria**

- `pnpm dev` in `apps/playground/` starts without runtime errors.  
- The worker loads the **real** `libs/wasm` bundle built via `../scripts/build-wasm.sh` and calls an exported function that returns geometry buffers (initially a trivial constant mesh such as a single triangle), with **no mocked WASM modules** in TypeScript.  
- TypeScript compiles in strict mode with **no `any` usages**, and ESLint runs cleanly (zero lint errors).

**Task 1.2 Current Status:**  
- `apps/playground` has a SvelteKit project with a Three.js scene manager, Web Worker, and WASM wrapper implemented; the worker calls `compile_and_render` from `libs/wasm` and the scene manager builds dynamic `THREE.BufferGeometry` from the returned `MeshHandle` (vertices + indices) instead of using hard-coded primitive geometry.  
- `pnpm check` and `pnpm build` pass.
- Integration with `libs/wasm` via the local `scripts/build-wasm.sh` workflow is verified.

**Task 2.4 Status (Partial):**
- Mesh wrapper updated to return structured diagnostics.
- Worker updated to use structured message protocol.
- UI diagnostics panel implemented.
- End-to-end flow verified via unit and integration tests in `apps/playground`.

---

### Task 1.3 – Local WASM Build Pipeline

**Goal**  
Build `libs/wasm` (and any other `wasm32-unknown-unknown` artifacts) locally using the Rust toolchain so developers can iterate without Docker.

**Steps**

1. **Use Local Rust Toolchain**  
   - Install the `wasm32-unknown-unknown` target and `wasm-bindgen-cli` locally (via `rustup` / `cargo install`).  
   - Ensure `WASI_SDK_PATH` is set when cross-compiling C(++) dependencies so build scripts can find clang/llvm.  
   - Build `libs/wasm` for the wasm target in release mode and run `wasm-bindgen` to emit artifacts into `libs/wasm/pkg`.

2. **`scripts/build-wasm.sh` Helper**  
   - Provide a Bash script that encapsulates the steps above: installs targets, verifies `wasm-bindgen`, respects `WASI_SDK_PATH`, runs `cargo build`, and invokes `wasm-bindgen`.  
   - The script writes JS/TS glue files and `.wasm` output into `libs/wasm/pkg`.

3. **Wire pnpm Scripts**  
   - Ensure `apps/playground/package.json` defines a `build:wasm` script that runs `../scripts/build-wasm.sh`.  
   - Optionally, have the main app `build` script depend on `build:wasm` (e.g. `"build": "pnpm build:wasm && vite build"`) so that the WASM bundle is always up to date before SvelteKit/Vite compilation.

**Acceptance Criteria**

- Running `pnpm build:wasm` in `apps/playground/` builds `libs/wasm` locally using the Rust toolchain (with optional WASI SDK).  
- Developers need a local `rustup`, `cargo`, and `wasm-bindgen` installation; Docker is no longer required for wasm builds.  
- The generated `.wasm` and JS/TS glue files are written into a stable location that the SvelteKit/Vite build can import.

**Task 1.3 Status:**
- `scripts/build-wasm.sh` implemented.
- `pnpm build:wasm` successfully builds `libs/wasm` locally and copies artifacts to `libs/wasm/pkg`.
- `apps/playground` consumes these artifacts via `$wasm` alias.

---

### Task 1.4 – Parser Infrastructure & Language Server (libs/openscad-lsp) 

**Goal**  
Provide a parse-only and structural analysis pipeline for OpenSCAD using `libs/openscad-parser`, and expose it via a Rust language server built with `tower-lsp`, without duplicating parser wiring in other crates.

**Steps**

1. **Create `libs/openscad-lsp` crate**  
   - Crate structure:  
     - `src/lib.rs` (public API for analysis).  
     - `src/server/mod.rs` (tower-lsp server setup).  
     - `src/document_store.rs` (in-memory text + versioning).  
     - `src/parser.rs` (Tree-sitter integration using `libs/openscad-parser` bindings).  
   - Add dependencies:  
     - `tower-lsp`.  
     - `tokio` (for async runtime).  
     - `libs/openscad-parser` as a workspace dependency.

2. **Tree-sitter Integration**  
   - Use `libs/openscad-parser/src/grammar.json` and the Rust bindings in `bindings/rust` (for example a `language()` function) to create and maintain a `tree_sitter::Parser` and `Tree`.  
   - Implement incremental parsing by applying `tree_sitter::InputEdit` on document changes and reparsing only affected regions.  
   - Add helpers to map positions between LSP `Position` and byte offsets/points used by Tree-sitter.

3. **High-Level Analysis API (parse-only)**  
   - Implement an internal API (for example `analyze_source(source: &str) -> Vec<Diagnostic>`) that:  
     - Parses the source with Tree-sitter.  
     - Collects syntax errors and basic structural issues into the shared `Diagnostic` type.  
     - Does **not** expose raw `tree_sitter::Node` values to callers; return only domain types (diagnostics, symbols, spans).

4. **tower-lsp Server Wiring**  
   - Implement an LSP server that:  
     - Handles `initialize`, `initialized`, `shutdown`, and `textDocument/*` requests.  
     - On `textDocument/didOpen` and `textDocument/didChange`, updates the document store and parser tree, then publishes diagnostics from `analyze_source`.  
   - Provide a thin `main.rs` (either in this crate or an `apps/openscad-lsp` binary) that starts the server over stdio.

5. **Parser Reuse Policy**  
   - Document that **all** IDE/editor-facing “parse-only” and structural analysis must go through `libs/openscad-lsp`.  
   - `libs/wasm` continues to own the runtime pipeline (`compile_and_render` etc.), but does **not** expose a separate `parse_only` entry point; avoid duplicating parser wiring there.  
   - Ensure the Playground and any external tools do **not** use `web-tree-sitter`; all parsing is Rust-based via `libs/openscad-parser`.

**Acceptance Criteria**

- A minimal `openscad-lsp` server binary can be launched (for example from an editor or CLI client) and responds correctly to `initialize` and `shutdown`.  
- Given a basic OpenSCAD snippet (e.g. `cube(10);`), the server publishes either zero diagnostics or a well-formed list of syntax diagnostics.  
- Parser integration lives only in `libs/openscad-parser` and `libs/openscad-lsp`; no other crate re-implements Tree-sitter wiring or depends on `web-tree-sitter`.

**Task 1.4 Status:**
- `libs/openscad-lsp` implemented with tower-lsp server and Tree-sitter integration.
- **Parser SRP Refactoring Completed** (see `specs/split-parser/` for details):
  - `libs/openscad-ast` refactored into modular architecture following SRP
  - Created focused modules: `statement.rs`, `module_call.rs`, `transform_chain.rs`, `assignments.rs`
  - Argument parsing split into `arguments/` submodules: `cube.rs`, `sphere.rs`, `cylinder.rs`, `shared.rs`
  - All modules under 500 lines with comprehensive documentation and co-located tests
  - 20 tests passing, zero regressions
  - Public API unchanged (`parse_to_ast` remains the entry point)
  - See `specs/split-parser/tasks.md` for complete refactoring breakdown

---

### Task 1.5 – Enforce Pipeline Boundaries 

**Goal**  
Ensure all future `libs/manifold-rs` implementations follow a consistent, mechanical C++ → Rust porting approach (index-based half-edge, `rayon` parallelism, explicit errors, robust predicates), before any primitives or boolean operations are added.

**Steps**

1. **Half-Edge Representation**  
   - Replace raw pointers or index fields that point into C++ arrays with **index-based handles** in Rust (`u32` indices into `Vec` arenas).  
   - Keep ownership in central arenas (e.g. `Vec<Vertex>`, `Vec<HalfEdge>`, `Vec<Face>`); pass indices between functions instead of references with complex lifetimes.

2. **Parallelism (thrust/TBB → `rayon`)**  
   - For C++ code that uses `thrust`/TBB to parallelize loops over faces/edges, map them to `par_iter()`/`par_iter_mut()` over the corresponding `Vec`s in Rust.  
   - Keep side effects confined to data local to each loop iteration, or use scoped parallelism patterns to avoid shared mutable state.

3. **Memory & Safety**  
   - Eliminate manual memory management patterns from C++ (new/delete, raw pointer arithmetic).  
   - Use `unsafe` only in small, well-audited sections where performance demands it, and always expose a safe API on top.  
   - Replace C++ `assert`/`abort` with explicit `Result`-based errors or internal debug assertions that never leak to the public API as panics.

4. **Error Handling**  
   - Convert C++ failure paths (error codes, special values) into typed Rust errors using `thiserror`.  
   - Ensure all public `manifold-rs` operations used by the Evaluator return `Result<Self, Error>` or similar, never relying on panics.

5. **Testing Strategy**  
   - Where possible, mirror C++ test cases/fixtures in Rust, comparing topological invariants (e.g. Euler characteristic, manifold validity) rather than relying only on exact floating-point equality.  
   - Add new tests that exercise edge cases surfaced by fuzzing and visual regression.

6. **Robust Predicates Initialization**  
   - If the chosen robust predicates library requires a one-time initialization (for example an `exactinit()` call), invoke this once at WASM startup (e.g. in a Rust `init` function or lazy static) so all downstream geometry code benefits from correct predicate behaviour.

**Example: Porting a Boolean Union Loop (Conceptual)**

- **C++ pattern (conceptual)**  
  - A typical C++ Manifold boolean operation iterates over `std::vector<Halfedge>` and `std::vector<Face>` collections, sometimes in parallel using `thrust`/TBB:
    - Build or update arrays of half-edges and faces.  
    - For each face/edge, classify it against a plane or another mesh and mark results in-place.  
    - Use parallel for-loops to accelerate classification and merging.

- **Rust pattern (mechanical translation)**  
  - Model the same data as arenas in `manifold-rs`:
    - `vertices: Vec<Vertex>`, `half_edges: Vec<HalfEdge>`, `faces: Vec<Face>` with `u32` indices connecting them.  
    - Each face stores indices into `half_edges`, each half-edge stores indices into `vertices` and adjacent faces.
  - Replace C++ loops with safe Rust iteration:
    - Sequential: `for face_idx in 0..faces.len() { let face = &mut faces[face_idx]; /* classify/update */ }`.  
    - Parallel: `faces.par_iter_mut().for_each(|face| { /* classify/update */ });` when no cross-face mutation is required (or when shared state is carefully managed).
  - Instead of mutating global flags or using raw pointers, have classification functions return explicit enums/results and update the arenas through indices.  
  - Any failure (numerical issues, invalid topology) must surface as a `Result` with a typed error, never as a silent assertion or abort.

**Template: C++ `Manifold::Boolean` → Rust `Manifold::boolean`**

- **C++ signature (from `manifold/src/manifold.cpp`)**  
  - `Manifold Manifold::Boolean(const Manifold& second, OpType op) const;`

- **Rust public API shape (in `libs/manifold-rs`)**  
  - Provide a safe, explicit API instead of operator overloading:
    - `pub fn boolean(&self, other: &Manifold, op: BooleanOp) -> Result<Manifold, BooleanError>;`  
    - `pub fn union(&self, other: &Manifold) -> Result<Manifold, BooleanError>;`  
    - `pub fn difference(&self, other: &Manifold) -> Result<Manifold, BooleanError>;`  
    - `pub fn intersection(&self, other: &Manifold) -> Result<Manifold, BooleanError>;`
  - `BooleanOp` is a small Rust enum mirroring `OpType` (`Add`, `Subtract`, `Intersect`).  
  - `BooleanError` is a `thiserror`-based type capturing triangulation failures, invalid topology, or precision/pathological cases.

- **Rust internal delegation pattern**  
  - Keep the user-facing `Manifold` as a thin handle around an internal implementation (e.g. `struct Manifold { impl_: Arc<Impl> }`).  
  - Implement the heavy boolean logic on an internal `Impl`/`Node` type that owns the index-based half-edge arenas.  
  - `Manifold::boolean` should:
    - Validate inputs (e.g. basic sanity checks, bounding boxes).  
    - Construct an internal CSG tree node (similar to `CsgNode`/`CsgOpNode` in C++) using indices and `rayon` for parallel phases.  
    - Execute the boolean operation, returning either a new `Impl` (wrapped in `Manifold`) or a `BooleanError`.

- **Testing pattern**  
  - Mirror the intention of `operator+`, `operator-`, `operator^` by:
    - Adding tests for `union/difference/intersection` that validate:  
      - Topological invariants (`validate()` passes, expected Euler characteristic).  
      - Expected behaviour on disjoint vs overlapping bounding boxes.  
    - Ensuring the Rust `boolean` operations never panic and always return a `Result`.

**Acceptance Criteria**

- Code reviews for new `libs/manifold-rs` features explicitly check against these guidelines (half-edge representation, parallelism, safety, error handling, testing, robust predicates).
- New public boolean APIs in `libs/manifold-rs` (`boolean`, `union`, `difference`, `intersection`) never panic in tests and always surface failures via `Result` with typed errors.
### Task 1.6 – WASM Runtime Packaging (Tree-sitter Style)

**Goal**  
Align the `libs/wasm` web distribution with Tree-sitter's `web-tree-sitter` pattern: a single JS entrypoint with a co-located `.wasm` binary and a clear async initialization API, while keeping build tooling Node-only.  

**Steps**

1. **Runtime Bundle Contract**  
   - Treat `libs/wasm/pkg` as the only browser-facing WASM distribution: it contains the wasm-bindgen glue (`wasm.js`) and the compiled binary (`wasm_bg.wasm`).  
   - Import this bundle in the Playground exclusively via the `$wasm` alias, never from build scripts or other ad-hoc paths.  

2. **TypeScript Wrapper Alignment**  
   - Ensure `apps/playground/src/lib/wasm/mesh-wrapper.ts` exposes an `initWasm(param?: WasmInitParameter)` function that mirrors `web-tree-sitter`'s `Parser.init()` pattern:  
     - Browser code calls `initWasm()` with no arguments and lets Vite resolve `wasm_bg.wasm`.  
     - Tests and Node code may pass explicit module bytes or a pre-fetched module for deterministic initialization.  

3. **Node-only Build Helpers**  
   - Keep `scripts/build-wasm.sh` and any Node-based helper (for example `build-wasm.js`) as CLI-only tools that are never imported into browser bundles.  
   - Document their intended usage in developer docs and verify no `apps/**` or `libs/**` runtime code imports these scripts.  

4. **Cross-platform Wiring**  
   - For Unix-like systems, ensure `pnpm build:wasm` in `apps/playground` calls `../../scripts/build-wasm.sh`.  
   - For Windows, provide an equivalent workflow (for example `pnpm build:wasm:win` calling `node ../../build-wasm.js`) if needed by contributors.  

**Acceptance Criteria**

- `libs/wasm/pkg/wasm.js` + `wasm_bg.wasm` are the only artifacts imported into browser-facing code, and they are initialized via `initWasm()` in a way analogous to `web-tree-sitter`.  
- All build helpers (`scripts/build-wasm.sh`, `build-wasm.js`, or equivalents) are clearly documented as Node-only and are not pulled into Vite/SvelteKit client bundles.  
- `pnpm build:wasm` (and any platform-specific variants) consistently regenerates `libs/wasm/pkg`, and `pnpm build` in `apps/playground` completes without bundler complaints about Node built-ins.  

### Task 2.1 – Manifold-RS Cube Primitive (TDD)

**Goal**  
Implement a robust cube primitive in `libs/manifold-rs` with TDD.

**Steps**

1. **Test First**
   - In `libs/manifold-rs/src/primitives/cube/tests.rs`:
     - Add tests asserting:
       - 8 vertices and 12 triangles for a simple cube.  
       - Correct bounding box for given `size` and `center` flag.  
       - `Manifold::validate()` passes.

2. **Implementation**
   - In `libs/manifold-rs/src/primitives/cube/mod.rs`:
     - Implement `pub fn cube(size: DVec3, center: bool) -> Manifold`.
     - Use the index-based half-edge representation.
   - Ensure cube construction relies on `Vec` arenas and u32 indices, not pointer-based structures.

3. **Robustness**
   - Where predicates are needed (e.g. coplanarity checks), use `robust`-style predicates from the beginning rather than ad-hoc epsilon comparisons.

4. **Validation**
   - Implement `Manifold::validate()` (if not already) to run Euler checks and topology invariants.

**Acceptance Criteria**

- `cargo test -p manifold-rs` passes with cube tests.
- `Manifold::Cube` (or equivalent) produces a valid manifold for typical sizes.

---

### Task 2.4 – Pipeline Integration & Error Reporting

**Goal**  
Connect source → CST → AST → evaluated/flattened AST → Mesh through the full pipeline for a minimal `cube(10);` program, and introduce structured diagnostics.

**Steps**

1. **Diagnostic Type**  
   - Define in a shared crate or module:

     ```rust
     struct Diagnostic {
         severity: Severity,
         message: String,
         span: Span,
         hint: Option<String>,
     }
     ```
   - This `Diagnostic` is the canonical error type used by `libs/openscad-parser`, `libs/openscad-ast`, `libs/openscad-eval`, and `libs/manifold-rs` when reporting problems (syntax errors, unsupported primitives, evaluation issues, etc.).  
   - `libs/wasm::diagnostics` provides a WASM-compatible `Diagnostic` wrapper that implements `From<openscad_ast::Diagnostic>` and exposes `severity()`, `message()`, `start()`, `end()`, and `hint()` getters for JavaScript.  
   - Downstream consumers (the WASM boundary and the Playground) must never invent diagnostics; they always originate from this shared type.

2. **Minimal `cube(10);` Pipeline Wiring**  
   - Implement a tracer-bullet path that exercises **every layer** as described in `overview-plan.md` §3.5:
     - Playground sends the source string `cube(10);` to `libs/wasm`.
     - `libs/wasm` forwards `cube(10);` into a single entry point in `libs/manifold-rs`.
     - `libs/manifold-rs` calls `libs/openscad-eval` with the original source string.
     - `libs/openscad-eval` calls `libs/openscad-ast` with `cube(10);`.
     - `libs/openscad-ast` calls `libs/openscad-parser` with `cube(10);`, receives a CST, converts it to a typed AST, and returns the AST to `libs/openscad-eval`.
     - `libs/openscad-eval` decides whether evaluation is required, evaluates and resolves the AST, and returns an evaluated/flattened AST to `libs/manifold-rs`.
     - `libs/manifold-rs` transforms the evaluated AST into a mesh and returns it to `libs/wasm`.
     - `libs/wasm` returns the mesh to the Playground as typed buffers/handles, and the Playground converts it into Three.js geometry and renders the cube.

3. **WASM Interface**  
   - In `libs/wasm`:
     - Implement an internal helper that returns either a mesh or a list of Rust diagnostics:

       ```rust
       pub fn compile_and_render_internal(
           source: &str,
       ) -> Result<MeshHandle, Vec<openscad_ast::Diagnostic>> {
           // Calls manifold-rs::from_source(source) and converts the result
           // into a MeshHandle on success, or returns a Vec<Diagnostic> on error.
       }
       ```

     - Implement a mapping function that converts Rust diagnostics into WASM-visible diagnostics:

       ```rust
       fn map_rust_diagnostics(
           diagnostics: Vec<openscad_ast::Diagnostic>,
       ) -> Vec<wasm::Diagnostic> {
           diagnostics
               .into_iter()
               .map(wasm::Diagnostic::from)
               .collect()
       }
       ```

     - Implement a helper that builds the JavaScript error payload containing a `diagnostics` array:

       ```rust
       fn build_diagnostics_error_payload(
           diagnostics: Vec<wasm::Diagnostic>,
       ) -> JsValue {
           use js_sys::{Array, Object, Reflect};
           use wasm_bindgen::JsValue;

           let array = Array::new();
           for diag in diagnostics {
               array.push(&JsValue::from(diag));
           }

           let obj = Object::new();
           Reflect::set(&obj, &JsValue::from_str("diagnostics"), &array)
               .expect("set diagnostics property");
           JsValue::from(obj)
       }
       ```

     - Expose the WASM entry point that uses `Result<MeshHandle, JsValue>` and the helpers above, aligning with the project’s Option B decision:

       ```rust
       #[wasm_bindgen]
       pub fn compile_and_render(source: &str) -> Result<MeshHandle, JsValue> {
           match compile_and_render_internal(source) {
               Ok(mesh) => Ok(mesh),
               Err(rust_diags) => {
                   let wasm_diags = map_rust_diagnostics(rust_diags);
                   let payload = build_diagnostics_error_payload(wasm_diags);
                   Err(payload)
               }
           }
       }
       ```

     - Ensure `MeshHandle` carries counts and typed vertex/index buffers suitable for building a `THREE.BufferGeometry` in the Playground, so the worker can return the real manifold mesh to the renderer.  
     - Do **not** add string-only or fallback error paths; all pipeline failures must flow through a structured `diagnostics` array.

4. **Playground Diagnostics**  
   - In the worker (`pipeline.worker.ts`), catch exceptions from the WASM wrapper and normalize them into a `compile_error` message that always carries a `diagnostics` array:

     ```ts
     try {
       const mesh = compile(source);
       (self as DedicatedWorkerGlobalScope).postMessage({
         type: 'compile_success',
         payload: mesh,
       });
     } catch (error: unknown) {
       const payload = error as { diagnostics?: Diagnostic[] };
       const diagnostics = payload.diagnostics ?? [];
       (self as DedicatedWorkerGlobalScope).postMessage({
         type: 'compile_error',
         payload: diagnostics,
       });
     }
     ```

   - In the Playground route (`+page.svelte`), handle `compile_error` messages by:
     - Updating status to an error state (for example `"Error"`).  
     - Storing the diagnostics in component state.  
     - Rendering a diagnostics panel that lists at least severity and message, and later uses `start`/`end` for source mapping.  
   - Do not crash or silently ignore diagnostics; invalid code must always produce visible, structured error information.

**Acceptance Criteria**

- Intentionally invalid OpenSCAD code produces a `Vec<Diagnostic>` with correct spans and messages in Rust, and the public `compile_and_render` binding throws a JavaScript error object of the form `{ diagnostics: Diagnostic[] }`.  
- The worker converts this error into a `compile_error` message whose `payload` is a non-empty diagnostics array, and the Playground renders these diagnostics in a panel instead of crashing.  
- A `cube(10);` snippet traverses the **full minimal pipeline** documented in `overview-plan.md` §3.5 (Playground → `libs/wasm` → `libs/manifold-rs` → `libs/openscad-eval` → `libs/openscad-ast` → `libs/openscad-parser` → back up to `libs/manifold-rs` → `libs/wasm` → Playground), verified by integration tests or targeted logging.

---

## Phase 3 – Filesystem & Parameters

### Task 3.1 – Named Arguments

**Goal**  
Support OpenSCAD-style named arguments (e.g. `cube(size=[10,10,10], center=true)`).

**Steps**

1. **Parser & AST Checks**
   - Ensure the parser/AST already supports named arguments; extend if necessary.

2. **Evaluator Argument Mapping**
   - Implement a helper to map positional + named args into a canonical structure for builtins like `cube`.

3. **Tests**
   - Add evaluator tests confirming that combinations of positional and named args produce the expected `GeometryNode::Cube` values.

**Acceptance Criteria**

- `cube(size=[10,10,10], center=true)` and `cube([10,10,10], true)` behave identically where appropriate.

---

### Task 3.2 – Resolution Special Variables ($fn, $fa, $fs)

**Goal**  
Represent and evaluate OpenSCAD’s resolution variables `$fn`, `$fa`, and `$fs` in a dedicated evaluation context.

**Steps**

1. **EvaluationContext Struct**  
   - `libs/openscad-eval::evaluator::context::EvaluationContext` tracks `$fn`, `$fa`, `$fs` alongside generic variables, using defaults from `config::constants`.
   - Expose getters/setters with doc comments and examples so future primitives can reuse the centralized configuration.

2. **Propagation**  
   - Parser/AST already emit `Statement::Assignment { name: "$fn" | "$fa" | "$fs" }`; the evaluator must route these into `EvaluationContext::set_variable`.  
   - Resolution-sensitive primitives (currently `sphere`) read from the context when constructing IR, applying the OpenSCAD formula:  
     - If `$fn > 0`, fragments = max($fn, 3).  
     - Otherwise, fragments = `ceil(min(360/$fa, 2πr/$fs))` with a lower bound of 5. *(References: OpenSCAD User Manual, Wikibooks “Circle resolution” section, and community write-ups on $fn/$fa/$fs.)*

3. **Tests**  
   - Evaluator regression tests now cover:  
     - Global `$fn` assignments affecting subsequent spheres.  
     - Per-call `$fn` overrides.  
     - `$fa/$fs` fallback when `$fn = 0`.  
   - Extend tests to future primitives as they gain resolution controls.

**Acceptance Criteria**

- `$fn`, `$fa`, `$fs` live in a documented context struct and are consumed by primitives per the OpenSCAD formulas cited above.  
- Automated tests prove changes in `$fn/$fa/$fs` alter generated `GeometryNode::Sphere` segments accordingly.

---

## Phase 4 – 2D Primitives & Extrusions

### Task 4.1 – 2D Kernel & Primitives
**Goal**  
Establish a 2D geometry representation in `libs/manifold-rs` and implement standard OpenSCAD 2D primitives (`square`, `circle`, `polygon`).

**Steps**
1. **2D Representation (`CrossSection`)**
   - Create `libs/manifold-rs/src/core/cross_section/{mod.rs, tests.rs}`.
   - Define a `CrossSection` struct that holds a list of contours (each contour is `Vec<Vec2>`).
   - Implement `from_contours` and `to_polygons` (triangulation using `earcutr` or similar for 3D conversion).
   - Ensure strict winding order enforcement (CCW for outer, CW for inner holes) to match OpenSCAD/Manifold expectations.

2. **Primitives Implementation**
   - **`square(size, center)`**:
     - Generate a rectangle contour.
     - Handle scalar vs vector `size` and `center` logic.
   - **`circle(r|d, $fn)`**:
     - Reuse `resolution::compute_segments` logic.
     - Generate a regular polygon contour.
   - **`polygon(points, paths)`**:
     - Parse points and paths.
     - Handle simple polygons (single path) and complex ones (multiple paths/holes).
     - Validate `convexity` parameter (store as metadata if needed, though mainly for CSG).

3. **Evaluator Integration**
   - Wire `GeometryNode::Square`, `GeometryNode::Circle`, `GeometryNode::Polygon` to `manifold-rs` 2D calls.
   - Ensure 2D shapes can be rendered (as flat 3D meshes on Z=0) for debug/preview.

**Acceptance Criteria**
- `square()`, `circle()`, `polygon()` render correctly in the playground (as flat plates).
- Vertex counts match OpenSCAD for equivalent `$fn`.
- `CrossSection` correctly handles holes (e.g. `difference()` in 2D - *note: 2D booleans might be needed later, for now just creation*).

---

### Task 4.2 – Linear Extrusion (Twist & Slices)
**Goal**  
Implement `linear_extrude` with full support for `height`, `twist`, `slices`, `center`, and `convexity`.

**Steps**
1. **Algorithm Implementation**
   - In `libs/manifold-rs/src/ops/extrude.rs`:
     - Implement `linear_extrude(cross_section, height, twist, slices, center, scale)`.
     - **Twist Logic**:
       - Calculate `angle_per_slice = twist / slices`.
       - Calculate `height_per_slice = height / slices`.
       - Loop `i` from 0 to `slices`:
         - Generate transformed contour: Rotate by `i * angle`, Translate Z by `i * height`.
         - Apply `scale` parameter if present (interpolated).
       - **Stitching**:
         - Create side quads (split to triangles) connecting indices `j` of slice `i` to `j` of slice `i+1`.
       - **Caps**:
         - Triangulate bottom cap (slice 0) and top cap (slice N).
         - Reverse winding for bottom cap.

2. **Evaluator Wiring**
   - Map `linear_extrude` arguments in `openscad-eval`.
   - Handle parameter validation (e.g. `slices` must be >= 1 if twist is used).

**Acceptance Criteria**
- `linear_extrude(height=10, twist=360, slices=50) square(10)` produces a twisted pillar.
- `center=true` correctly centers the mesh along Z.
- Matching behavior with OpenSCAD for twisted extrusions.

---

### Task 4.3 – Rotate Extrusion (Partial & Convexity)
**Goal**  
Implement `rotate_extrude` supporting `angle` (partial revolution) and `convexity`.

**Steps**
1. **Algorithm Implementation**
   - In `libs/manifold-rs/src/ops/revolve.rs`:
     - Implement `rotate_extrude(cross_section, angle, convexity, $fn)`.
     - **Full Revolution (360)**:
       - Use Manifold's native `Revolve` if available and robust.
     - **Partial Revolution (< 360)**:
       - Steps = `$fn` (or calculated from resolution vars).
       - Generate `steps + 1` profiles rotated around Z axis.
       - Stitch adjacent profiles to form the shell.
       - **Capping**: Explicitly triangulate the start profile (angle 0) and end profile (angle `a`).
       - Ensure proper winding for caps (start cap points "in", end cap points "out").

2. **Evaluator Wiring**
   - Map `rotate_extrude` parameters.
   - Ensure `$fn` is respected for the revolution steps.

**Acceptance Criteria**
- `rotate_extrude(angle=90) square(10)` produces a quarter-torus/washer.
- `angle=360` produces a closed torus.
- Caps are correctly generated for partial angles (watertight mesh).

---

## Phase 5 – Transformations & Modifiers

### Task 5.1 – Standard Transformations
**Goal**  
Implement `translate`, `rotate`, `scale`, `mirror`, `multmatrix`.

**Steps**
1. **Matrix Math**
   - Ensure `glam::DMat4` is used for all transformations.
   - Implement `multmatrix` by directly constructing a `DMat4` from the 4x4 input array.
   - `mirror`: Construct a reflection matrix (e.g. Identity - 2*v*v^T for normal v, or just scaling -1 along axis).

2. **Manifold Integration**
   - Expose `transform(mat)` on `Manifold` type.
   - Apply matrix to all vertices.
   - **Normals**: Apply inverse-transpose of the upper-left 3x3 for normals (if stored).

**Acceptance Criteria**
- `multmatrix` works for shear and non-uniform scaling.
- `mirror([1,0,0])` correctly flips the model.
- Chained transforms work as expected.

---

### Task 5.2 – Resize & Auto-Scaling
**Goal**  
Implement `resize([x,y,z], auto)` which scales geometry to fit a specific size.

**Steps**
1. **Bounding Box**
   - Ensure `Manifold::bounding_box()` returns accurate AABB.

2. **Resize Logic**
   - Calculate current size `curr = bbox.max - bbox.min`.
   - Determine target size:
     - If `auto` is true (or dimension is 0/undefined), scale factor should be uniform based on explicit dimensions to preserve aspect ratio.
     - If explicit, `scale = target / curr`.
   - Apply `scale([sx, sy, sz])`.

**Acceptance Criteria**
- `resize([10,0,0], auto=true) cube(5)` scales uniformly to X=10 (so Y=10, Z=10).
- `resize([10,5,2])` distorts the cube to 10x5x2.

---

### Task 5.3 – Color & Metadata
**Goal**  
Support `color(c)` to tag geometry with RGBA values, propagating through CSG.

**Steps**
1. **Data Structure**
   - Define `ScadObject` or `ManifoldWrapper`:
     ```rust
     pub struct ScadObject {
         pub mesh: Manifold,
         pub color: Option<Vec4>, // RGBA
         pub id: u32,
     }
     ```
   - Or use Manifold's `MeshRelation` / properties if viable, but a wrapper is simpler for OpenSCAD's volume-based color.

2. **CSG Propagation**
   - In `union(a, b)`:
     - OpenSCAD rule: "The result takes the color of the first operand".
     - Implementation: Result color = `a.color` (if present), else `b.color`.
     - *Advanced*: If OpenSCAD merges meshes preserving face colors, we need per-face attributes. For now, implement "volume color" (single color per Manifold).
     - *Refinement*: If `union` merges disparate colored objects, we might need to store color per-vert/face in Manifold. **Decision**: Implement per-face color metadata in Manifold (using `propVert` or `triProp`) to support multicolored unions.

**Acceptance Criteria**
- `color("red") cube(10)` renders red.
- `union() { color("red") cube(); color("blue") sphere(); }` renders parts with their respective colors (if per-face implemented) or first color (if volume-based). **Target per-face** for high fidelity.

---

### Task 5.4 – Offset (2D)
**Goal**  
Implement `offset(r|delta, chamfer)` for 2D shapes.

**Steps**
1. **Library Selection**
   - Use `cavalier_contours` (Rust Clipper port) for robust 2D offsetting.

2. **Implementation**
   - Convert `CrossSection` contours to Clipper paths.
   - Apply offset:
     - `r`: Round join, `delta`: Miter/Square join.
     - `chamfer`: Chamfer join.
   - Convert back to `CrossSection`.

**Acceptance Criteria**
- `offset(r=1) square(10)` produces a rounded square.
- `offset(delta=1, chamfer=true)` produces a chamfered square.

---

## Phase 6 – Advanced CSG & Hulls

### Task 6.1 – Boolean Operations
**Goal**  
Implement robust `union`, `difference`, `intersection` using Manifold kernel.

**Steps**
1. **Manifold Bindings**
   - Expose `Boolean3` operations.
   - Ensure robust handling of coplanar faces (Manifold handles this well).

2. **N-ary Operations**
   - `union()` often takes N children. Implement incremental or parallel reduction: `union(A, B, C) -> union(union(A, B), C)`.

**Acceptance Criteria**
- `difference() { cube(10); sphere(6); }` produces a cube with a spherical cutout.
- Water-tightness verified by `Manifold::validate()`.

---

### Task 6.2 – Hull (Convex Hull)
**Goal**  
Implement `hull()` for 2D and 3D sets of points.

**Steps**
1. **Algorithm**
   - Collect all vertices from all child meshes.
   - **3D**: Use `quickhull` crate (or port `chull`) to generate the convex hull of the point cloud.
   - **2D**: Use `graham_scan` or `monotone_chain` for 2D contours.

2. **Implementation**
   - `hull(A, B)`: Get verts(A) + verts(B) -> Compute Hull -> New Manifold.

**Acceptance Criteria**
- `hull() { cube([10,1,1]); translate([0,10,0]) cube([1,10,1]); }` produces a convex shape connecting the two bars.

---

### Task 6.3 – Minkowski Sum
**Goal**  
Implement `minkowski()` sum.

**Steps**
1. **Fast Path (Convex)**
   - If children are convex (or user assumes so), `minkowski(A, B) = hull(sum of all vertex pairs)`.
   - Implement vertex sum: `V_new = { v_a + v_b | v_a in A, v_b in B }`.
   - Compute Hull of `V_new`.

2. **General Case (Slow)**
   - Fallback (warn user or check convexity): Union of `A` translated by every vertex of `B`.
   - Use with caution; maybe set a limit on vertex count.

**Acceptance Criteria**
- `minkowski() { cube(10); sphere(1); }` produces a rounded cube.


---

## 7. Global Ongoing Tasks

These are continuous practices rather than one-time tasks:

- **Follow Current Rust Best Practices**  
  - Use modern Rust patterns (2021+/2024 editions), idiomatic error handling, and ownership patterns.

- **Browser & WASM Testing**  
  - Regularly run `wasm-pack test --headless --chrome` to validate behaviour in a browser-like environment.

- **Keep Docs in Sync**  
  - Update `overview-plan.md` and `tasks.md` whenever architecture or workflows change.  
  - Remove outdated sections instead of letting them drift.

- **Enforce SRP & File Size Limits**  
  - Refactor aggressively when a module grows too large or takes on multiple responsibilities.  
  - After each larger task, audit files approaching ~300–500 lines and, if needed, split logic into SRP-friendly submodules (e.g. `evaluator/calls`, `evaluator/transforms`).

- **Project Cleanup**  
  - Regularly remove unused code, APIs, and comments.  
  - Maintain consistent design patterns across crates.

- **Error Handling & Tests**  
  - In tests, assert on explicit error variants (from `thiserror`) instead of relying on panics or `unwrap`/`expect`.  
  - Treat any panic in library code paths as a bug to be removed.

- **Benchmarking & Fuzzing**  
  - Periodically benchmark end-to-end compile times and WASM size, adjusting algorithms and build flags (e.g. `wasm-opt`) as needed.  
  - Run fuzz tests for boolean operations regularly (e.g. nightly) and treat regressions as high severity.

This `tasks.md` should evolve with the project; treat it as a living backlog, always aligned with the codebase and the principles described in `overview-plan.md`.
