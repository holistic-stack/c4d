//! This crate provides OpenscadParser language support for the [tree-sitter][] parsing library.
//!
//! Typically, you will use the [LANGUAGE][] constant to add this language to a
//! tree-sitter [Parser][], and then use the parser to parse some code:
//!
//! ```
//! let code = r#"
//! "#;
//! let mut parser = tree_sitter::Parser::new();
//! let language = tree_sitter_openscad_parser::LANGUAGE;
//! parser
//!     .set_language(&language.into())
//!     .expect("Error loading OpenscadParser parser");
//! let tree = parser.parse(code, None).unwrap();
//! assert!(!tree.root_node().has_error());
//! ```
//!
//! [Parser]: https://docs.rs/tree-sitter/*/tree_sitter/struct.Parser.html
//! [tree-sitter]: https://tree-sitter.github.io/

use tree_sitter_language::LanguageFn;

extern "C" {
    fn tree_sitter_openscad() -> *const ();
}

/// The tree-sitter [`LanguageFn`][LanguageFn] for this grammar.
///
/// [LanguageFn]: https://docs.rs/tree-sitter-language/*/tree_sitter_language/struct.LanguageFn.html
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_openscad) };

/// The content of the [`node-types.json`][] file for this grammar.
///
/// [`node-types.json`]: https://tree-sitter.github.io/tree-sitter/using-parsers/6-static-node-types
pub const NODE_TYPES: &str = include_str!("../../src/node-types.json");

// NOTE: uncomment these to include any queries that this grammar contains:

// pub const HIGHLIGHTS_QUERY: &str = include_str!("../../queries/highlights.scm");
// pub const INJECTIONS_QUERY: &str = include_str!("../../queries/injections.scm");
// pub const LOCALS_QUERY: &str = include_str!("../../queries/locals.scm");
// pub const TAGS_QUERY: &str = include_str!("../../queries/tags.scm");

/// Parses OpenSCAD source code and returns a syntax tree.
///
/// This is a high-level wrapper around tree-sitter that handles parser
/// initialization and error reporting.
///
/// # Arguments
/// * `source` - The OpenSCAD source code to parse
///
/// # Returns
/// * `Ok(Tree)` - The parsed syntax tree if successful
/// * `Err(String)` - An error message if parsing fails
///
/// # Examples
/// ```
/// use tree_sitter_openscad_parser::parse_source;
///
/// let tree = parse_source("cube(10);").unwrap();
/// assert!(!tree.root_node().has_error());
/// ```
pub fn parse_source(source: &str) -> Result<tree_sitter::Tree, String> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&LANGUAGE.into())
        .map_err(|_| "Failed to load OpenSCAD grammar".to_string())?;
    
    parser
        .parse(source, None)
        .ok_or_else(|| "Parser returned None".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_load_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&LANGUAGE.into())
            .expect("Error loading OpenscadParser parser");
    }

    #[test]
    fn test_parse_simple_cube() {
        let tree = parse_source("cube(10);").expect("parse succeeds");
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_cube_with_vector() {
        let tree = parse_source("cube([1, 2, 3]);").expect("parse succeeds");
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_multiple_statements() {
        let source = "cube(10);\ncube([1, 2, 3]);";
        let tree = parse_source(source).expect("parse succeeds");
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let tree = parse_source("cube(").expect("parse returns tree");
        // Tree-sitter returns a tree even with errors
        assert!(tree.root_node().has_error());
    }
}
