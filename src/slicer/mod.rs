//! This module contains backend implementations to use specific slicer
//! implementation(s) to take a [crate::DesignFile] and produce gcode for
//! a specific make/model printer, given some config.

pub mod noop;
pub mod orca;
pub mod prusa;

use crate::{
    DesignFile, GcodeSlicer as GcodeSlicerTrait, GcodeTemporaryFile, ThreeMfSlicer as ThreeMfSlicerTrait,
    ThreeMfTemporaryFile,
};
use anyhow::Result;

/// All Slicers that are supported by the machine-api.
pub enum AnySlicer {
    /// Prusa Slicer
    Prusa(prusa::Slicer),

    /// Orca Slicer
    Orca(orca::Slicer),

    /// No-op Slicer -- only empty files!
    Noop(noop::Slicer),
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

impl From<noop::Slicer> for AnySlicer {
    fn from(slicer: noop::Slicer) -> Self {
        Self::Noop(slicer)
    }
}

impl GcodeSlicerTrait for AnySlicer {
    type Error = anyhow::Error;

    /// Generate gcode from some input file.
    async fn generate(&self, design_file: &DesignFile) -> Result<GcodeTemporaryFile> {
        match self {
            Self::Prusa(slicer) => GcodeSlicerTrait::generate(slicer, design_file).await,
            Self::Orca(slicer) => GcodeSlicerTrait::generate(slicer, design_file).await,
            Self::Noop(slicer) => GcodeSlicerTrait::generate(slicer, design_file).await,
        }
    }
}

impl ThreeMfSlicerTrait for AnySlicer {
    type Error = anyhow::Error;

    /// Generate gcode from some input file.
    async fn generate(&self, design_file: &DesignFile) -> Result<ThreeMfTemporaryFile> {
        match self {
            Self::Prusa(slicer) => ThreeMfSlicerTrait::generate(slicer, design_file).await,
            Self::Orca(slicer) => ThreeMfSlicerTrait::generate(slicer, design_file).await,
            Self::Noop(slicer) => ThreeMfSlicerTrait::generate(slicer, design_file).await,
        }
    }
}
