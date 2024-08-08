use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::fs::File;

/// A TemporaryFile wraps a normal [tokio::fs::File]`, but will attempt to
/// delete the file with this handle is dropped. File i/o can be done using
/// `as_mut` or `as_ref`.
pub struct TemporaryFile {
    inner: File,
    path: PathBuf,
}

impl TemporaryFile {
    /// Create a new TemporaryFile from an existing file on disk. When this
    /// struct is dropped, the path provdied will be unlinked from the
    /// filesystem.
    pub async fn new(path: &Path) -> Result<Self> {
        Ok(TemporaryFile {
            path: path.to_owned(),
            inner: File::open(path).await?,
        })
    }

    /// Return the file as a mutable borrow.
    pub fn as_mut(&mut self) -> &mut File {
        &mut self.inner
    }

    /// Return the file as an immutable borrow.
    pub fn as_ref(&self) -> &File {
        &self.inner
    }

    /// Return the path on the filesystem.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let path = self.path.clone();
        tokio::spawn(async {
            eprintln!("removing {}", path.display());
            tokio::fs::remove_file(path)
        });
    }
}
