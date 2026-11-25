//! What stops the server.

/// Why the server stopped: the connection failed, or the client's `initialize` wasn't LSP's.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    /// The connection to the client failed.
    #[error(transparent)]
    Protocol(#[from] lsp_server::ProtocolError),
    /// A message wasn't what LSP says it is.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
