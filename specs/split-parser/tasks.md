# Split Parser Tasks

## Task SP-1 – Establish Modular Parser Layout
- **Goal:** Create the folder structure and module skeletons described in `plan.md` without changing behaviour.
- **Steps:**
  1. Scaffold `parser/mod.rs`, `parser/statement.rs`, `parser/module_call.rs`, `parser/transform_chain.rs`, `parser/assignments.rs`, and `parser/arguments/{mod,cube,sphere,cylinder,shared}.rs` with module-level doc comments referencing the plan.
  2. Re-export existing functions (temporarily) so `parse_to_ast` still compiles while the body is moved incrementally.
  3. Add `parser/tests/mod.rs` with placeholder TODOs explaining which suites will live there.
- **Acceptance Criteria:**
  - New modules compile (even if they just `todo!()` temporarily) and total line counts per file stay <500.
  - `cargo test -p openscad-ast` still runs, proving no public API regressions.

## Task SP-2 – Migrate Primitive Argument Parsers (Cube/Sphere/Cylinder)
- **Goal:** Move all argument parsing helpers into `parser/arguments/*` modules and update call sites.
- **Steps:**
  1. Move `parse_cube_arguments` and friends to `arguments/cube.rs`, preserving doc comments and inline examples.
  2. Move `parse_sphere_arguments`, `parse_cylinder_arguments`, and shared helpers (`parse_f64`, `parse_u32`, `parse_bool`, vector parsing) into their dedicated files.
  3. Update `module_call.rs` to import the new helpers.
  4. Add/relocate unit tests so each arguments module has its own `#[cfg(test)]` block with the existing scenarios (cube scalar/vector, sphere radius errors, cylinder named/positional), ensuring they include doc examples per project rules.
- **Acceptance Criteria:**
  - `parser.rs` no longer defines argument helpers; everything lives in `parser/arguments/*`.
  - All associated tests pass and reside alongside the code they validate.
  - No behavioural regressions (confirmed via `cargo test -p openscad-ast`).

## Task SP-3 – Split Statement & Transform Logic
- **Goal:** Move `parse_statement`, `parse_transform_chain`, and `parse_var_declaration` into dedicated modules, keeping SRP.
- **Steps:**
  1. Implement `statement.rs` that owns top-level CST traversal (formerly `parse_statement`).
  2. Create `transform_chain.rs` housing `parse_transform_chain` and transform-specific helpers.
  3. Move assignment logic into `assignments.rs` (including diagnostics) with focused tests.
  4. Update `parser/mod.rs` to orchestrate parse-to-AST flow using the new modules.
- **Acceptance Criteria:**
  - Each module has clear doc comments and <= 500 lines.
  - `parser/mod.rs` contains only public API + module wiring.
  - Tests cover translate/rotate/scale nesting, assignment parsing, and error paths from their new locations.

## Task SP-4 – Consolidate Tests & Documentation
- **Goal:** Ensure test coverage lives near corresponding logic and document the new parser architecture.
- **Steps:**
  1. Move integration-style tests (multiple statements, nested transforms) into `parser/tests/*`.
  2. Add README-style comments in `parser/mod.rs` referencing `specs/split-parser/plan.md` and explaining how to extend the parser.
  3. Remove obsolete comments or duplication left over from the refactor.
- **Acceptance Criteria:**
  - All tests pass and are grouped logically (arguments vs transforms vs integration).
  - Documentation clearly describes the parser split so future work doesn’t regress into a monolith.
