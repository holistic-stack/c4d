//! Filesystem tests to ensure deterministic behavior.

use super::*;

/// Success path for reading existing files.
///
/// # Examples
/// ```
/// use openscad_eval::filesystem::{FileSystem, InMemoryFilesystem};
/// let mut fs = InMemoryFilesystem::default();
/// fs.insert("main.scad", "cube(1);");
/// assert_eq!(fs.read_to_string("main.scad").unwrap(), "cube(1);");
/// ```
#[test]
fn read_existing_file() {
    let mut fs = InMemoryFilesystem::default();
    fs.insert("main.scad", "cube(1);");
    assert_eq!(fs.read_to_string("main.scad").unwrap(), "cube(1);");
}

/// Failure scenario when file is missing.
#[test]
fn read_missing_file() {
    let fs = InMemoryFilesystem::default();
    let err = fs.read_to_string("missing.scad").unwrap_err();
    assert!(matches!(err, FileSystemError::NotFound { .. }));
}
