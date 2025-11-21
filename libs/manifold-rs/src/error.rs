use thiserror::Error;

/// Errors that can occur during manifold operations.
#[derive(Error, Debug)]
pub enum ManifoldError {
    /// The topology is invalid (e.g., open edges, non-manifold vertices).
    #[error("Invalid topology: {0}")]
    InvalidTopology(String),

    /// An index was out of bounds of the arena.
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(String),

    /// A generic error for when a boolean operation fails.
    #[error("Boolean operation failed: {0}")]
    BooleanError(String),
}
