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

    /// Return the path on the filesystem.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl AsMut<File> for TemporaryFile {
    fn as_mut(&mut self) -> &mut File {
        &mut self.inner
    }
}

impl AsRef<File> for TemporaryFile {
    fn as_ref(&self) -> &File {
        &self.inner
    }
}

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let path = self.path.clone();
        tokio::spawn(async {
            tracing::trace!(path = format!("{:?}", path), "removing dropped file");
            let _ = tokio::fs::remove_file(path).await;
        });
    }
}
