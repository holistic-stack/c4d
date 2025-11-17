use crate::span::Span;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SemanticKind {
    InvalidRange,
    InvalidDotIndex,
    MalformedBlock,
    EmptyParens,
    InvalidMatrix,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParseError {
    Syntax { message: String, span: Span },
    Semantic { kind: SemanticKind, span: Span, details: Option<String> },
    Unsupported { node_type: String, span: Span },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Syntax { message, .. } => write!(f, "{}", message),
            ParseError::Semantic { kind, .. } => write!(f, "semantic error: {:?}", kind),
            ParseError::Unsupported { node_type, .. } => write!(f, "unsupported: {}", node_type),
        }
    }
}

impl Error for ParseError {}