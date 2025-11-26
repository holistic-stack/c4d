//! # Language Server Implementation
//!
//! Tower-lsp based language server for OpenSCAD.

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

/// OpenSCAD Language Server implementation.
pub struct OpenScadLanguageServer {
    client: Client,
}

impl OpenScadLanguageServer {
    /// Creates a new language server instance.
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Parses a document and returns diagnostics.
    async fn parse_document(&self, uri: &Url, text: &str) {
        let mut parser = tree_sitter::Parser::new();
        let language = tree_sitter_openscad_parser::LANGUAGE;
        
        if parser.set_language(&language.into()).is_err() {
            return;
        }

        let tree = match parser.parse(text, None) {
            Some(t) => t,
            None => return,
        };

        let mut diagnostics = Vec::new();
        self.collect_errors(&tree.root_node(), text, &mut diagnostics);

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
    }

    /// Collects syntax errors from the tree.
    fn collect_errors(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if node.is_error() || node.is_missing() {
            let start = node.start_position();
            let end = node.end_position();

            let message = if node.is_missing() {
                format!("Missing: {}", node.kind())
            } else {
                let text = &source[node.start_byte()..node.end_byte().min(source.len())];
                format!("Syntax error near '{}'", text.chars().take(20).collect::<String>())
            };

            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: start.row as u32,
                        character: start.column as u32,
                    },
                    end: Position {
                        line: end.row as u32,
                        character: end.column as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message,
                ..Default::default()
            });
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_errors(&child, source, diagnostics);
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for OpenScadLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "OpenSCAD LSP initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.parse_document(&params.text_document.uri, &params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.first() {
            self.parse_document(&params.text_document.uri, &change.text)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        // Clear diagnostics when document is closed
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }
}
