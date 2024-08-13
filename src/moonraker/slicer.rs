use super::Client;
use crate::{DesignFile, Slicer as SlicerTrait, TemporaryFile};
use anyhow::Result;

impl SlicerTrait for Client {
    type Error = anyhow::Error;

    async fn generate(&self, design_file: &DesignFile) -> Result<TemporaryFile> {
        self.slicer.generate(design_file).await
    }
}
