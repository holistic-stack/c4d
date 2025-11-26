//! # Serialized CST Types
//!
//! Defines types for receiving a serialized Concrete Syntax Tree from JavaScript.
//! This allows parsing to happen in web-tree-sitter (browser) and AST conversion
//! to happen in Rust WASM.
//!
//! ## Architecture
//!
//! ```text
//! Browser: OpenSCAD Source → web-tree-sitter → Serialized CST (JSON)
//! WASM: Serialized CST → openscad-ast → AST → Geometry IR → Mesh
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use openscad_ast::cst::SerializedNode;
//! use openscad_ast::parse_from_cst;
//!
//! let cst: SerializedNode = serde_json::from_str(json)?;
//! let statements = parse_from_cst(&cst, source)?;
//! ```

use serde::{Deserialize, Serialize};

/// A serialized syntax tree node from web-tree-sitter.
///
/// This structure mirrors the JavaScript `SerializedNode` interface
/// defined in `openscad-parser.ts`.
///
/// # Fields
///
/// * `node_type` - The grammar rule name (e.g., "source_file", "module_call")
/// * `text` - The source text covered by this node
/// * `start_index` - Byte offset where this node starts
/// * `end_index` - Byte offset where this node ends
/// * `start_position` - Row/column position where this node starts
/// * `end_position` - Row/column position where this node ends
/// * `children` - All child nodes (including anonymous nodes)
/// * `named_children` - Only named child nodes
/// * `is_named` - Whether this is a named node in the grammar
/// * `field_name` - The field name if this node is a field child
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedNode {
    /// Node type from grammar (e.g., "source_file", "module_call", "number")
    #[serde(rename = "type")]
    pub node_type: String,

    /// Source text covered by this node
    pub text: String,

    /// Byte offset where this node starts
    #[serde(rename = "startIndex")]
    pub start_index: usize,

    /// Byte offset where this node ends
    #[serde(rename = "endIndex")]
    pub end_index: usize,

    /// Start position (row, column)
    #[serde(rename = "startPosition")]
    pub start_position: Position,

    /// End position (row, column)
    #[serde(rename = "endPosition")]
    pub end_position: Position,

    /// All child nodes
    pub children: Vec<SerializedNode>,

    /// Named children only
    #[serde(rename = "namedChildren")]
    pub named_children: Vec<SerializedNode>,

    /// Whether this is a named node
    #[serde(rename = "isNamed")]
    pub is_named: bool,

    /// Field name if this node is a field child
    #[serde(rename = "fieldName")]
    pub field_name: Option<String>,
}

/// Position in source code (row, column).
///
/// Both row and column are zero-based.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    /// Zero-based row number
    pub row: usize,
    /// Zero-based column number
    pub column: usize,
}

impl SerializedNode {
    /// Finds a child node by its type.
    ///
    /// # Arguments
    ///
    /// * `node_type` - The type to search for
    ///
    /// # Returns
    ///
    /// The first child with the matching type, or None.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let module_call = node.find_child("module_call");
    /// ```
    pub fn find_child(&self, node_type: &str) -> Option<&SerializedNode> {
        self.children.iter().find(|c| c.node_type == node_type)
    }

    /// Finds a named child by its type.
    ///
    /// # Arguments
    ///
    /// * `node_type` - The type to search for
    ///
    /// # Returns
    ///
    /// The first named child with the matching type, or None.
    pub fn find_named_child(&self, node_type: &str) -> Option<&SerializedNode> {
        self.named_children.iter().find(|c| c.node_type == node_type)
    }

    /// Finds a child by its field name.
    ///
    /// # Arguments
    ///
    /// * `field` - The field name to search for
    ///
    /// # Returns
    ///
    /// The child with the matching field name, or None.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let name = node.child_by_field("name");
    /// let value = node.child_by_field("value");
    /// ```
    pub fn child_by_field(&self, field: &str) -> Option<&SerializedNode> {
        self.children.iter().find(|c| c.field_name.as_deref() == Some(field))
    }

    /// Checks if this node is an error node.
    ///
    /// # Returns
    ///
    /// True if this node represents a syntax error.
    pub fn is_error(&self) -> bool {
        self.node_type == "ERROR"
    }

    /// Checks if this node is a missing node.
    ///
    /// # Returns
    ///
    /// True if this node was inserted by error recovery.
    pub fn is_missing(&self) -> bool {
        self.node_type.starts_with("MISSING")
    }

    /// Gets all children of a specific type.
    ///
    /// # Arguments
    ///
    /// * `node_type` - The type to filter by
    ///
    /// # Returns
    ///
    /// Vector of children with the matching type.
    pub fn children_by_type(&self, node_type: &str) -> Vec<&SerializedNode> {
        self.children.iter().filter(|c| c.node_type == node_type).collect()
    }

    /// Gets all named children of a specific type.
    ///
    /// # Arguments
    ///
    /// * `node_type` - The type to filter by
    ///
    /// # Returns
    ///
    /// Vector of named children with the matching type.
    pub fn named_children_by_type(&self, node_type: &str) -> Vec<&SerializedNode> {
        self.named_children.iter().filter(|c| c.node_type == node_type).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a test node for unit tests.
    fn test_node(node_type: &str, text: &str) -> SerializedNode {
        SerializedNode {
            node_type: node_type.to_string(),
            text: text.to_string(),
            start_index: 0,
            end_index: text.len(),
            start_position: Position { row: 0, column: 0 },
            end_position: Position { row: 0, column: text.len() },
            children: Vec::new(),
            named_children: Vec::new(),
            is_named: true,
            field_name: None,
        }
    }

    #[test]
    fn test_find_child() {
        let mut parent = test_node("source_file", "cube(10);");
        parent.children.push(test_node("module_call", "cube(10)"));
        
        let found = parent.find_child("module_call");
        assert!(found.is_some());
        assert_eq!(found.unwrap().node_type, "module_call");
    }

    #[test]
    fn test_child_by_field() {
        let mut parent = test_node("assignment", "x = 10");
        let mut name_node = test_node("identifier", "x");
        name_node.field_name = Some("name".to_string());
        parent.children.push(name_node);
        
        let found = parent.child_by_field("name");
        assert!(found.is_some());
        assert_eq!(found.unwrap().text, "x");
    }

    #[test]
    fn test_is_error() {
        let error_node = test_node("ERROR", "invalid");
        assert!(error_node.is_error());
        
        let normal_node = test_node("number", "10");
        assert!(!normal_node.is_error());
    }
}
