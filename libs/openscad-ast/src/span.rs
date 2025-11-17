use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Span {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start: Position,
    pub end: Position,
    pub text: Option<String>,
}

impl Span {
    pub fn from_node(source: &str, node: tree_sitter::Node) -> Self {
        let start_byte = node.start_byte();
        let end_byte = node.end_byte();
        let start_point = node.start_position();
        let end_point = node.end_position();
        let text = source.get(start_byte..end_byte).map(|s| s.to_string());
        Span {
            start_byte,
            end_byte,
            start: Position {
                row: start_point.row as usize,
                col: start_point.column as usize,
            },
            end: Position {
                row: end_point.row as usize,
                col: end_point.column as usize,
            },
            text,
        }
    }
}