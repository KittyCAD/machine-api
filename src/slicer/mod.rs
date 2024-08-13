//! This module contains backend implementations to use specific slicer
//! implementation(s) to take a [crate::DesignFile] and produce gcode for
//! a specific make/model printer, given some config.

pub mod orca;
pub mod prusa;

use crate::{DesignFile, TemporaryFile};
use anyhow::Result;

/// All Slicers that are supported by the machine-api.
pub enum AnySlicer {
    /// Prusa Slicer
    Prusa(prusa::Slicer),

    /// Orca Slicer
    Orca(orca::Slicer),
}

impl From<prusa::Slicer> for AnySlicer {
    fn from(slicer: prusa::Slicer) -> Self {
        Self::Prusa(slicer)
    }
}

impl From<orca::Slicer> for AnySlicer {
    fn from(slicer: orca::Slicer) -> Self {
        Self::Orca(slicer)
    }
}

impl AnySlicer {
    /// Generate gcode from some input file.
    pub async fn generate(&self, design_file: &DesignFile) -> Result<TemporaryFile> {
        match self {
            Self::Prusa(slicer) => slicer.generate(design_file).await,
            Self::Orca(slicer) => slicer.generate(design_file).await,
        }
    }
}
