//! This module contains backend implementations to use specific slicer
//! implementation(s) to take a [crate::DesignFile] and produce gcode for
//! a specific make/model printer, given some config.

mod config;
pub mod noop;
pub mod orca;
pub mod prusa;

use anyhow::Result;
pub use config::Config;

use crate::{
    DesignFile, GcodeSlicer as GcodeSlicerTrait, GcodeTemporaryFile, HardwareConfiguration,
    ThreeMfSlicer as ThreeMfSlicerTrait, ThreeMfTemporaryFile,
};

/// All Slicers that are supported by the machine-api.
#[non_exhaustive]
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
    async fn generate(
        &self,
        design_file: &DesignFile,
        hardware_configuration: &HardwareConfiguration,
    ) -> Result<GcodeTemporaryFile> {
        match self {
            Self::Prusa(slicer) => GcodeSlicerTrait::generate(slicer, design_file, hardware_configuration).await,
            Self::Noop(slicer) => GcodeSlicerTrait::generate(slicer, design_file, hardware_configuration).await,
            _ => Err(anyhow::anyhow!("slicer doesn't support gcode")),
        }
    }
}

impl ThreeMfSlicerTrait for AnySlicer {
    type Error = anyhow::Error;

    /// Generate gcode from some input file.
    async fn generate(
        &self,
        design_file: &DesignFile,
        hardware_configuration: &HardwareConfiguration,
    ) -> Result<ThreeMfTemporaryFile> {
        match self {
            Self::Prusa(slicer) => ThreeMfSlicerTrait::generate(slicer, design_file, hardware_configuration).await,
            Self::Orca(slicer) => ThreeMfSlicerTrait::generate(slicer, design_file, hardware_configuration).await,
            Self::Noop(slicer) => ThreeMfSlicerTrait::generate(slicer, design_file, hardware_configuration).await,
        }
    }
}
