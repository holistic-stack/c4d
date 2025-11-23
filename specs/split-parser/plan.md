# Split Parser Plan

## Background
- `libs/openscad-ast/src/parser.rs` currently contains *all* CST → AST conversion logic (statements, transforms, argument parsing, diagnostics, tests) in a single ~900 line file.
- This violates SRP, breaks the project rule that files must stay under 500 lines, and makes TDD difficult because even small changes require editing a monolith.
- Recent cylinder parsing work added even more responsibilities (radius normalization, resolution parameter decoding), highlighting the need for a modular structure.

## Goals
1. Restructure the parser into focused modules so each file owns a single concern and stays <500 lines.
2. Preserve existing parsing behaviour (cube, sphere, cylinder, transforms, assignments) while making it easier to add new primitives (polyhedron, etc.).
3. Keep tests co-located with the logic they validate (per module) to maintain TDD flow and shorten edit/build cycles.
4. Provide clear boundaries so future contributors know where to extend or modify behaviour.

## Proposed Structure
```
libs/openscad-ast/src/parser/
├── mod.rs                # Public entry point (parse_to_ast, shared exports)
├── statement.rs          # Statement dispatch + top-level traversal
├── module_call.rs        # Primitive parsing (cube, sphere, cylinder, future polyhedron)
├── transform_chain.rs    # translate/rotate/scale handling
├── assignments.rs        # Variable assignments + diagnostics
├── arguments/
│   ├── mod.rs            # Re-exports
│   ├── cube.rs           # cube() argument parsing + helpers
│   ├── sphere.rs         # sphere() argument parsing + helpers
│   ├── cylinder.rs       # cylinder() argument parsing + helpers
│   └── shared.rs         # Shared numeric parsing, bool parsing, vector helpers
└── tests/
    ├── mod.rs            # Integration tests covering cross-module flows
    ├── primitives.rs     # cube/sphere/cylinder specific tests
    └── transforms.rs     # translate/rotate/scale regression tests
```
- `parse_to_ast` (public API) will live in `parser/mod.rs`, orchestrating tree-sitter invocation, error collection, and delegating to `statement::parse_statement`.
- Each arguments module includes doc comments + examples per project rules.
- Shared helpers (e.g., `parse_f64`, `parse_u32`, `parse_bool`, `parse_vector`) will move into `arguments/shared.rs` so they can be reused without circular deps.

## TDD & Process
1. **Baseline tests** – snapshot existing behaviour by keeping current `parser.rs` tests running before refactor.
2. **Module extraction** – move one responsibility at a time (e.g., cube arguments) into its own module with the existing tests relocated alongside it.
3. **Integration layer** – once all functions move, `parser/mod.rs` re-exports the same API so downstream crates see no change.
4. **SRP enforcement** – ensure every new file remains <500 lines and only owns a single responsibility.
5. **Documentation** – add module-level comments + examples for each new file.

## Risks & Mitigations
- *Risk:* Accidentally changing parsing behaviour while moving code.  
  *Mitigation:* Move functions verbatim and rely on existing tests; add targeted tests when splitting.
- *Risk:* Circular dependencies between modules.  
  *Mitigation:* Keep shared helpers in `arguments/shared.rs` and pass data via plain structs/enums from `ast_types`.
- *Risk:* Build breaks mid-refactor.  
  *Mitigation:* Work in small steps (one module at a time) with TDD, committing only when tests pass.

## Success Criteria
- No file in the parser folder exceeds 500 lines.
- All existing parser tests (including new cylinder cases) pass without behavioural regressions.
- New structure is documented and discoverable via `specs/split-parser/tasks.md`.
