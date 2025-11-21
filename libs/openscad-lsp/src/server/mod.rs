use crate::document_store::DocumentStore;
use crate::parser::OpenscadParser;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

pub struct Backend {
    pub client: Client,
    pub document_store: Mutex<DocumentStore>,
    pub parser: Mutex<OpenscadParser>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_store: Mutex::new(DocumentStore::new()),
            parser: Mutex::new(OpenscadParser::new()),
        }
    }

    async fn on_change(&self, uri: Url, text: String) {
        self.document_store.lock().await.update(uri.clone(), text.clone());
        
        let mut parser = self.parser.lock().await;
        let tree = parser.parse(&text);
        
        let diagnostics = if let Some(tree) = tree {
            let root = tree.root_node();
            if root.has_error() {
                // Simple error detection for now: find error nodes
                // In a real implementation, we'd traverse the tree to find specific error nodes
                vec![Diagnostic {
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 1),
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("openscad-lsp".to_string()),
                    message: "Syntax error detected".to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                }]
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
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
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_change(params.text_document.uri, params.text_document.text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // For FULL sync, the first content change contains the full text
        if let Some(change) = params.content_changes.first() {
            self.on_change(params.text_document.uri, change.text.clone()).await;
        }
    }
}
