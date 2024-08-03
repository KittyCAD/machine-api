//! A trait for a slicer.

pub mod orca;
pub mod prusa;

use anyhow::Result;

/// A slicer interface.
#[async_trait::async_trait]
pub trait Slicer: Send + Sync {
    /// Slice a file.
    /// Returns the path to the sliced file.
    async fn slice(&self, file: &std::path::Path) -> Result<std::path::PathBuf>;
}
