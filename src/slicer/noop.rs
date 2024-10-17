//! `noop` implements a no-op Machine, one that will accept Control commands
//! and do exactly nothing with it.

use anyhow::Result;

use crate::{
    traits::MachineSlicerInfo, DesignFile, GcodeSlicer as GcodeSlicerTrait, GcodeTemporaryFile, TemporaryFile,
    ThreeMfSlicer as ThreeMfSlicerTrait, ThreeMfTemporaryFile,
};

/// Noop-slicer won't slice anything at all!
#[derive(Copy, Clone, Debug)]
pub struct Slicer {}

impl Slicer {
    /// Create a new No-op Slicer. It won't do anything.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Slicer {
    fn default() -> Self {
        Self::new()
    }
}

impl GcodeSlicerTrait for Slicer {
    type Error = anyhow::Error;

    async fn generate(&self, _design_file: &DesignFile) -> Result<GcodeTemporaryFile> {
        let filepath = std::env::temp_dir().join(format!("{}", uuid::Uuid::new_v4().simple()));
        {
            let _ = std::fs::File::create(&filepath);
        }
        Ok(GcodeTemporaryFile(TemporaryFile::new(&filepath).await?))
    }
}

impl ThreeMfSlicerTrait for Slicer {
    type Error = anyhow::Error;

    async fn generate(
        &self,
        _design_file: &DesignFile,
        _machine_info: &MachineSlicerInfo,
    ) -> Result<ThreeMfTemporaryFile> {
        let filepath = std::env::temp_dir().join(format!("{}", uuid::Uuid::new_v4().simple()));
        {
            let _ = std::fs::File::create(&filepath);
        }
        Ok(ThreeMfTemporaryFile(TemporaryFile::new(&filepath).await?))
    }
}
