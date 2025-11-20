# tree-sitter-openscad

A [tree-sitter](https://tree-sitter.github.io/tree-sitter/) parser for OpenSCAD.

## Usage

### Rust

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
tree-sitter = "0.20"
tree-sitter-openscad = { path = "path/to/libs/openscad-parser" }
```

Example usage:

```rust
use tree_sitter::{Parser, Language};

fn main() {
    let mut parser = Parser::new();
    let language = tree_sitter_openscad::language();
    parser.set_language(language).expect("Error loading OpenSCAD grammar");

    let source_code = "cube([10, 10, 10]);";
    let tree = parser.parse(source_code, None).unwrap();
    let root_node = tree.root_node();

    println!("{}", root_node.to_sexp());
}
```

### TypeScript / JavaScript

#### Installation

```bash
npm install tree-sitter-openscad
```

#### Usage

```javascript
const Parser = require('tree-sitter');
const OpenSCAD = require('tree-sitter-openscad');

const parser = new Parser();
parser.setLanguage(OpenSCAD);

const sourceCode = 'cube([10, 10, 10]);';
const tree = parser.parse(sourceCode);

console.log(tree.rootNode.toString());
```

### WASM

To use in a web environment (like the Playground), you will typically use `web-tree-sitter`.

1. Build the WASM binary:
   ```bash
   tree-sitter build-wasm
   ```
2. Load it in your application:
   ```javascript
   const Parser = require('web-tree-sitter');
   await Parser.init();
   const parser = new Parser();
   const Lang = await Parser.Language.load('tree-sitter-openscad.wasm');
   parser.setLanguage(Lang);
   ```

## Development

### Running Tests

```bash
npm test
```

### Generating Grammar

If you modify `grammar.js`, regenerate the parser:

```bash
npm run generate
```

### Syntax Highlighting

Queries for syntax highlighting are located in `queries/highlights.scm`.

## Supported Syntax

The parser supports the full OpenSCAD language, including:

- **Variables**: Assignments, special variables (`$fn`, `$fa`, etc.).
- **Modules**: Definitions, instantiation, nested modules, `children()`.
- **Functions**: Definitions, literals, recursive functions.
- **Expressions**: Binary/unary operators, ranges, list comprehensions, ternary operators.
- **Control Flow**: `if`, `for`, `intersection_for`, `let`, `assign`.
- **Built-ins**: `echo`, `render`, `assert`, `import`, `include`, `use`.
- **Modifiers**: `*` (disable), `!` (root), `#` (debug), `%` (background).
- **2D/3D Primitives**: `cube`, `sphere`, `cylinder`, `square`, `circle`, `polygon`, `text`.
- **Transformations**: `translate`, `rotate`, `scale`, `mirror`, `multmatrix`, `color`, `offset`, `hull`, `minkowski`.
