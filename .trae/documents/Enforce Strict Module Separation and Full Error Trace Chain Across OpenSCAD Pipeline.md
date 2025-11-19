## Objectives
- Implement strict module boundaries: parsing (openscad-parser) → AST (openscad-ast) → evaluation (openscad-eval) → geometry/kernel (manifold-rs) → interface (libs/wasm).
- Ensure parsing entry is the parser layer (openscad-parser as the parse/CST provider); never call openscad-ast directly for parsing outside openscad-eval.
- Implement consistent, stage-aware error wrapping with spans and file context; serialize the full cause chain at the WASM boundary.
- Update documentation to reflect module responsibilities, public APIs, and the error propagation chain.

## Module Requirements & Implementations
### 1) openscad-ast
- Internals: Use the generated Rust binding from `libs/openscad-parser/bindings/rust` to parse and build the CST; encapsulate CST parsing.
- Public API: Expose only AST (non-evaluated) builders:
  - `fn build_ast_from_source(source: &str, file: Option<&str>) -> Result<Ast, AstError>`
  - Optionally `fn build_ast(cst: &Cst) -> Result<Ast, AstError>` for internal use
- Error Handling: Use `thiserror`-based `AstError` with `Span`, `file`, and `source()` to wrap underlying `ParseError` (from parser layer).
- Constraints: Never expose `parse_cst` publicly; CST parsing logic remains internal.

### 2) openscad-eval
- Internals: Call `openscad-ast::build_ast_from_source` to get AST; evaluate AST into Evaluated IR.
- Public API:
  - `fn parse_and_evaluate(source: &str, file: Option<&str>, ctx: &EvaluationContext) -> Result<EvaluatedAst, EvalError>`
  - `fn evaluate(ast: &Ast, ctx: &EvaluationContext) -> Result<EvaluatedAst, EvalError>`
- Error Handling: `EvalError` wraps `AstError` via `#[from]` with stage metadata; includes `Span`, `file`, and preserves `source()` chain.
- Constraints: Clean interface; no leaking of CST/AST internals.

### 3) manifold-rs
- Internals: Consume `EvaluatedAst` and perform geometry operations; export meshes.
- Public API:
  - `fn compile(source: &str, file: Option<&str>, ctx: &EvaluationContext) -> Result<Mesh, KernelError>` (calls parse+eval internally)
  - `fn build_geometry(ir: &EvaluatedAst) -> Result<Geometry, KernelError>`
  - `fn to_mesh(geom: &Geometry) -> Result<Mesh, KernelError>`
- Error Handling: `KernelError` wraps `EvalError` with stage info; carries spans and file; `source()` points to underlying cause.
- Constraints: Strict separation of evaluation vs geometric operations; efficient algorithms per kernel-decision.

### 4) libs/wasm
- Interface-only: Expose WASM functions that orchestrate via `manifold-rs` only.
- Public API:
  - `#[wasm_bindgen] pub fn parse_only(source: &str) -> Result<(), JsValue>` → calls `manifold_rs::compile` with a dry-run parse/eval mode or dedicated `manifold_rs::parse_only`; returns serialized chained diagnostics on error.
  - `#[wasm_bindgen] pub fn compile_and_render(source: &str) -> Result<MeshHandle, JsValue>` → calls `manifold_rs::compile`; returns typed buffer handle and frees memory when done.
- Error Serialization: Walk `Error::source()` chain to build `Vec<TraceDiagnostic>` `{ stage, message, span, file, hint }` and serialize via `serde_wasm_bindgen` or JSON.
- Constraints: No direct use of `openscad-ast` or parser; handle WASM/native conversions, enforce memory lifecycle and browser performance.

## Error Propagation Chain
- Stages: `Parse` (openscad-parser), `Ast` (openscad-ast), `Eval` (openscad-eval), `Kernel` (manifold-rs), `Wasm` (libs/wasm).
- Each stage defines an error type implementing `std::error::Error` via `thiserror`, with fields:
  - `stage: &'static str`, `message: String`, `span: Span { start, end }`, `file: Option<String>`, `hint: Option<String>`
- Wrapping Rules:
  - Parser returns `ParseError` with precise spans and file; AST wraps as `AstError(source = ParseError)`; Eval wraps as `EvalError(source = AstError)`; Kernel wraps as `KernelError(source = EvalError)`.
  - WASM converts the chain into `TraceDiagnostic[]` preserving order and context.
- Acceptance: Final WASM error must include original source location, parse failure details, intermediate steps, and final manifestation.

## Tests
- Unit tests per boundary:
  - Parser binding (doctest) loads grammar; invalid code returns errors with spans.
  - openscad-ast: invalid CST wraps errors correctly; produces AST for valid source.
  - openscad-eval: invalid AST evaluation wraps upstream errors; evaluates valid AST.
  - manifold-rs: `compile` wraps upstream errors; geometry and mesh invariants validated.
  - libs/wasm: `parse_only`/`compile_and_render` produce full trace diagnostics; memory lifecycle tests.
- Integration tests:
  - Malformed source (e.g., `cube(`) yields Parse→Ast→Eval→Kernel chain; spans preserved.
  - Valid source renders expected buffers; zero-copy verified.

## Documentation Updates
- `specs/pipeline/overview-plan.md`:
  - Clarify modules and strict boundaries; set parser layer as `openscad-parser` (Tree-sitter + Rust binding), AST consumes bindings internally.
  - Add error propagation chain section with stage definitions and example payload.
  - Update dependency graph: `playground → wasm → manifold-rs → openscad-eval → openscad-ast → openscad-parser`.
- `specs/pipeline/grammar-decision.md`:
  - Emphasize runtime parsing via AST layer using `openscad-parser` bindings; no direct TS parsing.
  - Note error spans preserved across CST→AST→Eval.
- `specs/pipeline/kernel-decision.md`:
  - Reinforce that manifold-rs exclusively owns geometry and mesh export; error wrapping at kernel stage with upstream chain.
- `specs/pipeline/tasks.md`:
  - Add a task “Error Chain Enforcement” with acceptance criteria for full trace diagnostics and module separation.

## Migration Steps
1. Add `thiserror` error types per crate with spans, file, hint; implement `From` conversions.
2. Hide CST parsing in openscad-ast; consume `libs/openscad-parser` bindings internally.
3. Implement `parse_and_evaluate` in openscad-eval; ensure no direct parser exposure.
4. Add `compile` and `parse_only` in manifold-rs that only call eval; remove any direct AST usage outside eval.
5. Update libs/wasm to call manifold-rs only; implement trace extraction and serialization.
6. Write unit/integration tests; run `cargo test` and playground tests.
7. Update all docs with the new boundaries and error chain.

## Acceptance Criteria
- No module exposes internal implementation details; all communications via public interfaces.
- libs/wasm never calls openscad-ast or parser directly; uses manifold-rs.
- Error handling produces a full chain with spans and file context; WASM outputs complete diagnostics.
- Performance and memory management optimized across the pipeline; tests and docs updated accordingly.