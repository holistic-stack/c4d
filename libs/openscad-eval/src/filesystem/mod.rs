//! Filesystem abstractions used by the evaluator.
//!
//! The initial tracer bullet relies on an in-memory implementation to satisfy
//! the "no mocks" constraint even during tests.

use std::collections::HashMap;
use thiserror::Error;

/// Minimal filesystem trait supporting OpenSCAD source retrieval.
///
/// # Examples
/// ```
/// use openscad_eval::filesystem::{FileSystem, InMemoryFilesystem};
/// let fs = InMemoryFilesystem::default();
/// assert!(fs.read_to_string("main.scad").is_err());
/// ```
pub trait FileSystem {
    /// Reads an entire file into memory.
    fn read_to_string(&self, path: &str) -> Result<String, FileSystemError>;
}

/// Error raised when filesystem operations fail.
///
/// # Examples
/// ```
/// use openscad_eval::filesystem::{FileSystem, FileSystemError, InMemoryFilesystem};
/// let fs = InMemoryFilesystem::default();
/// let err = fs.read_to_string("foo.scad").unwrap_err();
/// match err {
///     FileSystemError::NotFound { .. } => {}
/// }
/// ```
#[derive(Debug, Error, PartialEq)]
pub enum FileSystemError {
    /// The requested path could not be found.
    #[error("file not found: {path}")]
    NotFound { path: String },
}

/// In-memory filesystem intended for tests and WASM usage.
///
/// # Examples
/// ```
/// use openscad_eval::filesystem::{FileSystem, InMemoryFilesystem};
/// let mut fs = InMemoryFilesystem::default();
/// fs.insert("scene.scad", "cube(1);");
/// assert_eq!(fs.read_to_string("scene.scad").unwrap(), "cube(1);");
/// ```
#[derive(Debug, Default, Clone)]
pub struct InMemoryFilesystem {
    files: HashMap<String, String>,
}

impl InMemoryFilesystem {
    /// Inserts or replaces a file entry.
    pub fn insert(&mut self, path: impl Into<String>, contents: impl Into<String>) {
        self.files.insert(path.into(), contents.into());
    }
}

impl FileSystem for InMemoryFilesystem {
    fn read_to_string(&self, path: &str) -> Result<String, FileSystemError> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| FileSystemError::NotFound { path: path.into() })
    }
}

#[cfg(test)]
mod tests;
