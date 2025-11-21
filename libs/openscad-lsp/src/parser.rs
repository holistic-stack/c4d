use tree_sitter::{Parser, Tree};
use tree_sitter_openscad_parser::LANGUAGE;

pub struct OpenscadParser {
    parser: Parser,
}

impl OpenscadParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(&LANGUAGE.into())
            .expect("Error loading OpenSCAD grammar");
        Self { parser }
    }

    pub fn parse(&mut self, text: &str) -> Option<Tree> {
        self.parser.parse(text, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_initialization() {
        let _parser = OpenscadParser::new();
    }

    #[test]
    fn test_parse_simple_cube() {
        let mut parser = OpenscadParser::new();
        let tree = parser.parse("cube([10, 10, 10]);");
        assert!(tree.is_some());
        let tree = tree.unwrap();
        let root = tree.root_node();
        assert!(!root.has_error());
    }
}

impl Default for OpenscadParser {
    fn default() -> Self {
        Self::new()
    }
}

