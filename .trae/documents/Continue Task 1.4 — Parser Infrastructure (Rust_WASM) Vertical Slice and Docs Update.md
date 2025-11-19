## Scope & Objectives
- Continue the next logical backlog item: Task 1.4 — Parser Infrastructure (Rust/WASM) in `specs/pipeline/tasks.md:116-140`.
- Ensure parsing happens entirely in Rust/WASM; remove any TS-side `web-tree-sitter` usage.
- Introduce a shared `Diagnostic` type across crates and expose a WASM `parse_only` entry.
- Update documentation to reflect vertical slice progress and decisions.

## Current State Review
- Kernel strategy confirmed as a direct port (`specs/pipeline/kernel-decision.md`).
- Grammar variant confirmed to keep local grammar (`specs/pipeline/grammar-decision.md`).
- Overview plan requires WASM functions returning diagnostics (`specs/pipeline/overview-plan.md:124-131, 154-169`).
- Playground setup (Task 1.2) is in progress and ready to call WASM (`specs/pipeline/tasks.md:90-113`).

## Implementation Steps
1. Parser Crate Wiring
- Verify `libs/openscad-parser` is in the workspace and consumed by `libs/openscad-ast` with Rust bindings (`bindings/rust/lib.rs`).
- Confirm CST→AST conversion preserves `Span { start, end }` as per `overview-plan.md:153-169`.
- Ensure no TS import of `web-tree-sitter` or parser WASM assets in Playground (Task 1.4 requirement).

2. Shared Diagnostic Type
- Create a minimal shared crate for pipeline types (e.g., `libs/pipeline-types`) to host `Span` and `Diagnostic` to avoid dependency cycles:
  - `pub struct Span { start: usize, end: usize }`
  - `pub struct Diagnostic { severity: Severity, message: String, span: Span, hint: Option<String> }`
- Reference this crate from `openscad-ast`, `openscad-eval`, `manifold-rs`, and `wasm` per the dependency graph (`overview-plan.md:173-184`).

3. WASM Entry Point (Parsing Only)
- In `libs/wasm`, expose `#[wasm_bindgen] pub async fn parse_only(source: &str) -> Result<(), JsValue>` that internally returns `Result<(), Vec<Diagnostic>>` and maps diagnostics via `serde_wasm_bindgen`.
- Initialize `console_error_panic_hook` in debug builds (`overview-plan.md:129`).

4. Playground Worker Integration
- In the worker, call `parse_only(source)` as the sole parsing path; remove any `web-tree-sitter` usage.
- Define a strict TS message contract (no `any`), kebab-case filenames, and forward `Diagnostic[]` to UI.

5. Tests & QA (TDD, No Mocks except I/O)
- Rust unit tests: CST→AST preserves spans; invalid inputs return `Diagnostic` with correct ranges.
- WASM integration tests via `wasm-pack test --headless --chrome` (`tasks.md:472-474`).
- Playground e2e: worker receives diagnostics and UI shows squiggles.
- CI additions: `cargo fmt`, `cargo clippy`, `cargo test`, wasm tests, Playwright e2e (`overview-plan.md:374-381`).

6. Performance & WASM Best Practices (2025)
- Enable size and speed optimizations in release:
  - `lto = true`, `opt-level = 's'|'z'`, `panic = 'abort'`, consider `codegen-units = 1`.
  - Post-process with `wasm-opt -O4` or `-Os` for additional shrink and speed [RustWasm team guidance](https://github.com/rustwasm/team/issues/109) and [Rust & WebAssembly book](https://rustwasm.github.io/book/reference/code-size.html).
- Threading readiness for future `rayon` in WASM:
  - Build with `+atomics,+bulk-memory,+mutable-globals` and use `--target web` when enabling threads; requires `SharedArrayBuffer` and COOP/COEP headers [Parallel Raytracing guide](https://rustwasm.github.io/docs/wasm-bindgen/examples/raytrace.html), [wasm‑bindgen‑rayon context](https://www.reddit.com/r/rust/comments/m7stbw/wasmbindgenrayon_an_adapter_for_enabling/).
  - Note browser constraints and shared memory requirements [discussion](https://www.reddit.com/r/rust/comments/12rn5fq/state_of_web_assembly_and/).

7. Documentation Updates
- Update `specs/pipeline/tasks.md`:
  - Mark Task 1.4 as In Progress → Completed after tests; list steps, and acceptance criteria met.
- Update `specs/pipeline/overview-plan.md`:
  - Confirm Rust-backed parsing path via `libs/wasm` entry; document `Diagnostic` crate and worker contract.
- Keep `grammar-decision.md` and `kernel-decision.md` unchanged (already confirmed), but cross-link from the Overview.

## Acceptance Criteria
- Worker calls `libs/wasm` parse entry and returns success or `Diagnostic[]` for inputs like `cube(10);` (`tasks.md:136-140`).
- No `web-tree-sitter` or parser WASM assets in Playground; parsing occurs in Rust/WASM.
- Tests pass locally and in CI; diagnostics correctly show spans.

## Risks & Mitigation
- Dependency cycles: Mitigated by a tiny shared `pipeline-types` crate.
- WASM threading complexity: Plan for `rayon` only when COOP/COEP and `SharedArrayBuffer` are enabled; keep current slice single-threaded.
- Binary size: Apply `lto`, `opt-level`, `panic=abort`, and `wasm-opt` in release builds.

## Follow-Ups (Next Slices)
- Task 2.4 — Pipeline Integration & Error Reporting: Wire `compile_and_render` with diagnostics end-to-end (`specs/pipeline/tasks.md:255-287`).
- Task 3.1 — Named Arguments: Implement evaluator mapping and tests (`specs/pipeline/tasks.md:292-311`).
- Task 3.2 — Resolution Variables: Add `EvaluationContext` for `$fn/$fa/$fs` (`specs/pipeline/tasks.md:315-334`).