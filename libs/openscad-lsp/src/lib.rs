//! # OpenSCAD Language Server
//!
//! Language Server Protocol implementation for OpenSCAD using tower-lsp.
//!
//! ## Features
//!
//! - Syntax diagnostics via tree-sitter
//! - Document symbols
//! - Incremental parsing
//!
//! ## Usage
//!
//! ```rust,ignore
//! use openscad_lsp::run_server;
//!
//! #[tokio::main]
//! async fn main() {
//!     run_server().await;
//! }
//! ```

pub mod server;

pub use server::OpenScadLanguageServer;

/// Runs the language server over stdio.
pub async fn run_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = tower_lsp::LspService::new(|client| {
        OpenScadLanguageServer::new(client)
    });

    tower_lsp::Server::new(stdin, stdout, socket)
        .serve(service)
        .await;
}
