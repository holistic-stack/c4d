## Test Methodology

* Use Tree-sitter corpus tests per "Writing Tests" guidance: name between `====`, input, separator `---`, expected S-expression. Include attributes like `:error`, `:fail-fast`, `:cst` where useful.

* Validate both successful parses and invalid inputs. Add field names (`name:`, `arguments:`, `body:`) to improve clarity.

* Organize by complexity: `basic/`, `intermediate/`, `advanced/`. The CLI runs all `.txt` under `test/corpus`.

## Current Parser Highlights

* Entry: `source_file` handles `use_statement` and items libs\openscad-parser\grammar.js:125.

* Statements include `transform_chain`, control-flow, `union_block`, and `include_statement` libs\openscad-parser\grammar.js:143–157.

* Object creation via `transform_chain` → `module_call` → trailing `statement` libs\openscad-parser\grammar.js:253–263.

* Modules/functions: `module_item`, `function_item` libs\openscad-parser\grammar.js:183–199.

* Expressions: unary/binary/ternary, function/index/dot-index, `let_expression` libs\openscad-parser\grammar.js:128–141,339–377.

* Literals: `string`, `number` (`integer`/`float`), `boolean`, `undef`, `range`, `list` libs\openscad-parser\grammar.js:276–301,432–447.

* Special variables: `$` + identifier libs\openscad-parser\grammar.js:406–408.

## Directory Layout to Add

* `libs/openscad-parser/test/corpus/basic/`

* `libs/openscad-parser/test/corpus/intermediate/`

* `libs/openscad-parser/test/corpus/advanced/`

## Basic Tests (files to add)

* `basic/primitives.txt`

  * Cube, sphere, cylinder calls with positional and named args; semicolon termination.

  * Example:

```
==================
Primitive shapes — cube/sphere/cylinder
==================

cube(10);
sphere(d=20);
cylinder(h=5, r1=2, r2=3);

---

(source_file
  (transform_chain
    (module_call
      name: (identifier)
      arguments: (arguments (number)))
    (statement))
  (transform_chain
    (module_call
      name: (identifier)
      arguments: (arguments
        (assignment name: (identifier) value: (number))))
    (statement))
  (transform_chain
    (module_call
      name: (identifier)
      arguments: (arguments
        (assignment name: (identifier) value: (number))
        (assignment name: (identifier) value: (number))
        (assignment name: (identifier) value: (number))))
    (statement)))
```

* `basic/transformations.txt`

  * `translate([x,y,z]) cube(5);`, `rotate([a,b,c]) sphere(5);`, `scale([sx,sy,sz]) cylinder(h=10, r=2);`.

  * Verify nested `transform_chain` structure.

* `basic/variables.txt`

  * Simple declarations: identifiers, strings, ranges, special variables.

  * Example:

```
==================
Variable declaration and assignment
==================

a = 1;
$fn = 50;
name = "OpenSCAD";
r = [0:1:10];

---

(source_file
  (var_declaration (assignment name: (identifier) value: (number)))
  (var_declaration (assignment name: (special_variable (identifier)) value: (number)))
  (var_declaration (assignment name: (identifier) value: (string)))
  (var_declaration (assignment name: (identifier)
    value: (range start: (number) increment: (number) end: (number)))))
```

* `basic/includes.txt`

  * `use <lib.scad>;`, `include <lib.scad>;` libs\openscad-parser\grammar.js:205–213.

* `basic/literals.txt`

  * Strings with escapes, booleans, integers, floats with exponents, `undef`.

## Intermediate Tests (files to add)

* `intermediate/booleans.txt`

  * `union() { cube(10); sphere(5); }`

  * `difference() { cube(10); sphere(5); }`

  * `intersection() { cube(10); cylinder(h=10, r=3); }`

  * Expect `transform_chain` with `module_call` and `union_block` body containing child `transform_chain`s.

* `intermediate/modules.txt`

  * `module m(a, b=2, $fn=32) { translate([a,b,0]) cube(a); }` libs\openscad-parser\grammar.js:183–189.

  * Trailing commas in `parameters` libs\openscad-parser\grammar.js:171–176.

* `intermediate/loops-and-conditionals.txt`

  * `for (i = [0:1:2]) cube(i);`

  * `intersection_for (i = [1:3]) sphere(i);` libs\openscad-parser\grammar.js:231–235.

  * `if (true) cube(1); else sphere(2);` libs\openscad-parser\grammar.js:246–251.

* `intermediate/math-and-functions.txt`

  * `function add(x, y) = x + y;` libs\openscad-parser\grammar.js:190–199.

  * `a = cos(PI/4) + 1;` (function\_call, binary\_expression, identifier).

* `intermediate/indexing.txt`

  * `v = [1,2,3]; v[0]; v.z;` (index\_expression, dot\_index\_expression) libs\openscad-parser\grammar.js:344–351.

* `intermediate/assert-echo.txt`

  * `assert(true) cube(1);`

  * `assert(true);` (optional expression) libs\openscad-parser\grammar.js:391–396.

  * `echo("value", 1) sphere(1);` libs\openscad-parser\grammar.js:399–403.

## Advanced Tests (files to add)

* `advanced/hierarchies.txt`

  * `# color("red") translate([1,2,3]) rotate([0,0,90]) scale([2,2,2]) cube(1);` (modifiers + nested `transform_chain`) libs\openscad-parser\grammar.js:253–258.

* `advanced/recursive-functions.txt`

  * `function fact(n) = n <= 1 ? 1 : n * fact(n-1);` (ternary + recursive call) libs\openscad-parser\grammar.js:373–377.

  * `function fib(n) = n <= 1 ? n : fib(n-1) + fib(n-2);`

* `advanced/special-vars-and-operators.txt`

  * `$children`, `$preview`, `$fa`, `$fs`, `$t` usage in assignments and expressions libs\openscad-parser\grammar.js:406–408.

  * Unary ops `!`, `+`, `-`; binary precedence (`||`, `&&`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `+`, `-`, `*`, `/`, `%`, `^`) libs\openscad-parser\grammar.js:352–372.

* `advanced/extrusions-and-projection.txt`

  * `linear_extrude(height=10, twist=720, slices=200) rotate_extrude(angle=180) circle(10);`

  * `projection(cut=true) union() { cube(10); sphere(5); }`

* `advanced/comprehensions.txt`

  * `[for (i = [0:10]) i]`, `[for (i = [0:10]) if (i % 2 == 0) i]`, `[each [1,2,3]]` libs\openscad-parser\grammar.js:314–336.

## Error Tests (files to add)

* `basic/errors.txt`

  * `:error` Missing semicolon after module call: `cube(10)`.

  * `:error` Invalid identifier: `2x = 1;`.

  * `:error` Unclosed string: `name = "abc;`.

* `intermediate/errors.txt`

  * `:error` Malformed parameters: `module m(,){}`.

  * `:error` Invalid `if` condition: `if () cube(1);`.

  * `:error` Bad range: `[0::10]`.

* `advanced/errors.txt`

  * `:error` Unbalanced brackets in nested transforms.

  * `:error` Bad ternary: `a = cond ? 1 : ;`.

  * `:error` Invalid dot index: `list . 1;`.

## Naming Conventions

* Test names describe the construct and intent; one behavior per entry.

* Use field names in expected trees where defined (`name:`, `arguments:`, `body:` etc.).

## Running Locally

* In `libs/openscad-parser`, install Tree-sitter CLI and run tests:

  * `npm i -D tree-sitter-cli`

  * `npx tree-sitter test` (or `tree-sitter test -i 'Test Name'` to filter).

* Existing Rust sanity test remains (`bindings/rust/lib.rs`) and can run via `cargo test -p openscad-parser`.

## CI/CD Integration

* Add a GitHub Actions workflow `.github/workflows/openscad-parser-tests.yml` to:

  * Run on Ubuntu (Makefile warns Windows unsupported).

  * Steps: checkout → setup Node → install `tree-sitter-cli` → run `npx tree-sitter test` in `libs/openscad-parser` → setup Rust toolchain → `cargo test -p openscad-parser`.

* Optionally add a `package.json` script in `libs/openscad-parser`:

  * `"test:corpus": "tree-sitter test"` for consistency.

## Acceptance Criteria

* Corpus covers primitives, transforms, variables; control-flow, modules, booleans, math, functions; advanced hierarchies, recursion, special variables, transformations, comprehensions.

* Each file includes at least one `:error` entry validating invalid input handling.

* `npx tree-sitter test` passes on Linux/macOS; CI runs corpus and Rust loader tests.

* Tests reference actual node names and fields from the current grammar and validate structure per node-types (`src/node-types.json`).

## Next Step

* I will create the `test/corpus` directories and add the outlined

