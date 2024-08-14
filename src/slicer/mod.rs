//! This module contains backend implementations to use specific slicer
//! implementation(s) to take a [crate::DesignFile] and produce gcode for
//! a specific make/model printer, given some config.

pub mod orca;
pub mod prusa;

use crate::{DesignFile, GcodeSlicer as GcodeSlicerTrait, TemporaryFile, ThreeMfSlicer as ThreeMfSlicerTrait};
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

impl GcodeSlicerTrait for AnySlicer {
    type Error = anyhow::Error;

    /// Generate gcode from some input file.
    async fn generate(&self, design_file: &DesignFile) -> Result<TemporaryFile> {
        match self {
            Self::Prusa(slicer) => GcodeSlicerTrait::generate(slicer, design_file).await,
            Self::Orca(slicer) => GcodeSlicerTrait::generate(slicer, design_file).await,
        }
    }
}

impl ThreeMfSlicerTrait for AnySlicer {
    type Error = anyhow::Error;

    /// Generate gcode from some input file.
    async fn generate(&self, design_file: &DesignFile) -> Result<TemporaryFile> {
        match self {
            Self::Prusa(slicer) => ThreeMfSlicerTrait::generate(slicer, design_file).await,
            Self::Orca(slicer) => ThreeMfSlicerTrait::generate(slicer, design_file).await,
        }
    }
}
