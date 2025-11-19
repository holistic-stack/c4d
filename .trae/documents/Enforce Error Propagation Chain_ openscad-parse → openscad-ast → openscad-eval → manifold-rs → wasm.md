## Objectives
- Make `libs/wasm/src/lib.rs` use parsing exclusively through the designated chain: openscad-parse → openscad-ast → openscad-eval → manifold-rs → wasm.
- Never call `openscad-ast` from `manifold-rs` or `wasm` directly.
- Preserve full error trace (root cause, spans, stage names) through all layers and expose it at the WASM boundary.

## Current State Review
- `libs/wasm/src/lib.rs` calls `manifold_rs::process_openscad` for compile, good.
- `parse_only_internal` calls `manifold_rs::parse_only`, and `manifold-rs::parse_only` currently calls `openscad_ast::parse_cst` directly (violates rule).
- `openscad-eval::evaluate` already orchestrates CST → AST → Evaluated AST.

## Design: Error Types and Traceability
- Introduce explicit error types with `thiserror` and `source` chaining across crates. Each error stores stage, message, `Span`, optional file path, and optional hint.
- Stages: `ParseStage`, `AstStage`, `EvalStage`, `KernelStage`, `WasmStage`.
- Add a common `TraceDiagnostic` type in `pipeline-types`:
  - `{ stage: String, message: String, span: Span, file: Option<String>, hint: Option<String>, causes: Option<Vec<TraceDiagnostic>> }`
- Conversions: In each layer, wrap and forward errors with `#[from]` and attach stage metadata and spans; preserve `source` chain.

## Refactor Steps
1. Create `libs/openscad-parse`
- Extract CST parsing from `openscad-ast` into a new crate with public `parse_cst(source: &str, file: Option<&str>) -> Result<Cst, ParseError>`.
- `Cst` is a thin typed wrapper around tree-sitter outputs and spans.

2. Update `openscad-ast`
- Depend on `openscad-parse` for CST.
- Public: `build_ast(cst: &Cst) -> Result<Ast, AstError>`.
- No direct parsing; error wrapping with stage `AstStage` and spans.

3. Update `openscad-eval`
- Public: `parse_and_evaluate(source: &str, file: Option<&str>, ctx: &EvaluationContext) -> Result<EvaluatedAst, EvalError>`.
- Internally: `openscad-parse::parse_cst` → `openscad-ast::build_ast` → evaluation; wrap errors (`EvalError`) with `source` and stage.
- Add `parse_only(source, file) -> Result<(), EvalError>` that runs parse plus AST build, for diagnostics.

4. Update `manifold-rs`
- Remove any direct `openscad-ast` usage.
- Public:
  - `parse_only(source: &str, file: Option<&str>) -> Result<(), KernelError>` → delegates to `openscad_eval::parse_only` and wraps errors as `KernelError`.
  - `compile(source: &str, file: Option<&str>) -> Result<Mesh, KernelError>` → parse+eval, then geometry build and mesh export.
- Wrap evaluation errors with `source` chain and stage metadata.

5. Update `libs/wasm`
- `parse_only(source)` calls `manifold_rs::parse_only(source, None)` only; never touches `openscad-ast`.
- On `Err(e)`, traverse `Error::source()` chain to build a `Vec<TraceDiagnostic>` with stages and spans; serialize (JSON or via `serde_wasm_bindgen`).
- `compile_and_render(source)` calls `manifold_rs::compile`, returns handle/buffers; on error, same trace extraction.

## Error Trace Extraction (WASM layer)
- Implement utility to walk `std::error::Error` sources:
  1. Start at top-level error (`KernelError`), add its diagnostic.
  2. Loop over `source()` to collect each inner error (`EvalError`, `AstError`, `ParseError`).
  3. Preserve `Span`, optional `file`, stage names and messages; include `hint` where present.
- Serialize to `TraceDiagnostic[]` for the final JS consumer to display.

## Tests
- Unit tests per crate:
  - `openscad-parse`: invalid code returns `ParseError` with correct span and file.
  - `openscad-ast`: improper CST produces `AstError` wrapping `ParseError`.
  - `openscad-eval`: invalid AST yields `EvalError` with `source` chain.
  - `manifold-rs`: `parse_only`/`compile` wrap upstream errors as `KernelError` and preserve chain.
  - `libs/wasm`: `parse_only` returns a serialized chain where top-level stage is `Kernel` and inner stages include `Eval`, `Ast`, `Parse` with spans.
- Integration test: `cube(` produces a trace chain Parse→Ast→Eval→Kernel→WASM.

## Documentation
- Update `overview-plan.md`:
  - Explicit error flow and stage mapping.
  - Rule: `openscad-ast` must not be called outside `openscad-eval`.
- Update `tasks.md` with a new task “Error Chain Enforcement” and acceptance criteria for traceability.
- Add examples of error JSON payload (TraceDiagnostic[]) and display mapping.

## Acceptance Criteria
- `libs/wasm` uses `manifold-rs` only; no direct `openscad-ast` usage.
- Full error chain (root cause, spans, file, stage names) propagated to WASM and serialized.
- All unit and integration tests pass.
- Documentation explains the chain and enforcement rules.
