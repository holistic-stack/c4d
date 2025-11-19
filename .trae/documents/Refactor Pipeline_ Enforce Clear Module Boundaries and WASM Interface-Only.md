## Goals
- Enforce strict separation across parsing, AST, evaluation, geometry, and WASM interface.
- libs/wasm becomes a thin WebAssembly interface. All mesh generation and geometry processing lives in manifold-rs.
- Establish clear public interfaces and tests at each boundary, with end-to-end integration tests.
- Update documentation with pipeline diagrams, public APIs, version compatibility, and examples.

## Architectural Changes
### Crate Structure & Responsibilities
1. openscad-parse (rename from current `openscad-parser`)
- Responsibility: Parse OpenSCAD source → Concrete Syntax Tree (CST)
- Public API: `fn parse_cst(source: &str) -> Result<Cst, Vec<Diagnostic>>`
- Notes: Cst is a typed wrapper around tree-sitter outputs; preserves spans.

2. openscad-ast
- Responsibility: CST → AST (non-evaluated)
- Public API: `fn build_ast(cst: &Cst) -> Result<Ast, Vec<Diagnostic>>`
- Constraints: No evaluation logic.

3. openscad-eval
- Responsibility: AST → Evaluated AST (Geometry IR)
- Public API: `fn evaluate(ast: &Ast, ctx: &EvaluationContext) -> Result<EvaluatedAst, Vec<Diagnostic>>`
- Notes: Introduce `EvaluationContext` for `$fn`, `$fa`, `$fs` and global params.

4. manifold-rs
- Responsibility: Evaluated AST → Geometry & Mesh export
- Public APIs:
  - `fn build_geometry(ir: &EvaluatedAst) -> Result<Geometry, Error>`
  - `fn to_mesh(geom: &Geometry) -> Result<Mesh, Error>`
  - `fn export_gl(mesh: &Mesh) -> GlMeshBuffers` (flat f32 buffers + indices)
  - Booleans & transforms: `union/difference/intersection`, `apply_transform`
- Constraints: Mirrors C++ Manifold features and robustness.

5. libs/wasm
- Responsibility: Interface-only orchestration.
- Public APIs:
  - `fn parse_only(source: &str) -> Result<(), JsValue>` (already present; keep)
  - `async fn compile_and_render(source: &str) -> Result<MeshHandle, JsValue>`
- Implementation: Orchestrates calls to openscad-parse → openscad-ast → openscad-eval → manifold-rs; no mesh logic inside libs/wasm.
- MeshHandle: pointer/lengths to WASM memory with `free_*` to release.

### Dependency Graph (strict)
playground → wasm → manifold-rs → openscad-eval → openscad-ast → openscad-parse
- No back-edges; no cross-leakage of internal types. All communication via public interfaces.

## Refactor Plan
1. Crate rename & interface normalization
- Rename `libs/openscad-parser` to `libs/openscad-parse`.
- Add `parse_cst` returning a typed `Cst` with spans; wrap tree-sitter nodes.

2. AST boundary cleanup
- `openscad-ast` consumes `Cst` and outputs `Ast` only; remove any evaluation-like logic.

3. Evaluation boundary
- `openscad-eval` consumes `Ast` and returns `EvaluatedAst` (Geometry IR), with `EvaluationContext`.

4. Geometry & mesh consolidation
- Move all mesh generation/export from libs/wasm into `manifold-rs`.
- Provide `build_geometry`, `to_mesh`, `export_gl` APIs.

5. WASM interface
- Ensure `libs/wasm` only orchestrates and exposes public functions. Remove any internal mesh generation.
- Keep `parse_only` as Rust-backed parsing entry (no `web-tree-sitter` in TS).

## Public Interface Specs
- openscad-parse
  - Types: `Cst`, `Span`, `Diagnostic`
  - API: `parse_cst(&str) -> Result<Cst, Vec<Diagnostic>>`
- openscad-ast
  - Types: `Ast`
  - API: `build_ast(&Cst) -> Result<Ast, Vec<Diagnostic>>`
- openscad-eval
  - Types: `EvaluationContext`, `EvaluatedAst`
  - API: `evaluate(&Ast, &EvaluationContext) -> Result<EvaluatedAst, Vec<Diagnostic>>`
- manifold-rs
  - Types: `Geometry`, `Mesh`, `GlMeshBuffers`
  - APIs: `build_geometry(&EvaluatedAst) -> Result<Geometry, Error>`, `to_mesh(&Geometry) -> Result<Mesh, Error>`, `export_gl(&Mesh) -> GlMeshBuffers`
- libs/wasm
  - Types: `MeshHandle` (pointers/lengths), `Diagnostic`
  - APIs: `parse_only(&str) -> Result<(), JsValue>`, `compile_and_render(&str) -> Result<MeshHandle, JsValue>`

## Tests
### Unit Tests (per boundary)
- openscad-parse: valid/invalid syntax → `Cst`/diagnostics; span coverage.
- openscad-ast: CST → AST mapping; ensure no evaluation; spanning preserved.
- openscad-eval: AST → EvaluatedAst with `$fn/$fa/$fs` context; named/positional params.
- manifold-rs: geometry primitives (cube, sphere) produce valid meshes; `validate()` invariants; robust predicates on edge cases.
- libs/wasm: `parse_only` success/error; `compile_and_render` returns handles; memory lifecycle tests.

### Integration Tests
- End-to-end: source → CST → AST → EvaluatedAst → Geometry → Mesh → `GlMeshBuffers` → WASM handle.
- Golden tests for common models (cube, sphere) to validate mesh hashes match C++ Manifold expectations.

## Documentation Updates
- Update `specs/pipeline/overview-plan.md` with the enforced pipeline and dependency graph.
- Update `specs/pipeline/tasks.md` with refactor tasks and acceptance criteria.
- Add diagrams (ASCII initially; later PNG/SVG) showing data flow: `OpenSCAD → CST → AST → EvaluatedAst → Geometry → Mesh → WASM`.
- Public interface docs (Rust doc comments) with examples compiling under tests.
- Version compatibility matrix (crate versions; Rust edition 2024; wasm-bindgen version; SvelteKit/Vite versions).
- Examples of end-to-end chain from source to `GlMeshBuffers`.

## Constraints & Enforcement
- Strict module boundaries: use newtypes to prevent leakage of internal structs; keep conversions at API boundaries.
- No cross-module internal imports; depend only on public types.
- CI checks: `cargo deny` (optional), `cargo clippy` lints, forbid `any` in TypeScript, file-size limits.

## Performance & QA
- Syntax coverage: confirm against `libs/openscad-parse/test/corpus/**`.
- Backward compatibility: keep manifold-rs behavior aligned with C++ Manifold; validate via invariants.
- Benchmarks: measure end-to-end compile times; apply `lto`, `opt-level='s'|'z'`, `panic='abort'`, and `wasm-opt` in release.
- Mesh quality: compare triangle counts, watertight checks, Euler checks vs C++ Manifold outputs.

## Migration Steps
1. Introduce `openscad-parse` crate (rename) and `parse_cst` API.
2. Refactor `openscad-ast` to consume `Cst` and output `Ast` (remove evaluation traces).
3. Refactor `openscad-eval` to output `EvaluatedAst` only.
4. Move mesh logic to `manifold-rs`; expose `build_geometry/to_mesh/export_gl` APIs.
5. Update `libs/wasm` to orchestrate only; keep `parse_only` and add `compile_and_render` returning `MeshHandle`.
6. Update playground TypeScript wrapper to strictly consume `MeshHandle` and free buffers.
7. Add tests at all boundaries; add integration tests; update CI.
8. Update docs and diagrams.

## Acceptance Criteria
- libs/wasm contains no mesh generation logic; only orchestration and public functions.
- Each crate exposes documented public interfaces; tests validate boundaries.
- End-to-end compile succeeds; integration tests pass; playground renders geometry via zero-copy buffers.
- Documentation updated with diagrams, interface specs, compatibility, and examples.
- Performance and mesh quality validated against C++ Manifold standards.
