#![allow(refining_impl_trait)]
#![deny(missing_docs)]
#![deny(missing_copy_implementations)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unused_import_braces)]
#![deny(unused_qualifications)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]

//! This crate implements support for taking designed parts, and producing
//! real-world constructions of those parts.

mod any_machine;
#[cfg(feature = "bambu")]
pub mod bambu;
mod discover;
mod file;
#[cfg(feature = "formlabs")]
pub mod formlabs;
pub mod gcode;
mod machine;
#[cfg(feature = "moonraker")]
pub mod moonraker;
pub mod noop;
pub mod server;
pub mod slicer;
mod sync;
#[cfg(test)]
mod tests;
mod traits;
#[cfg(feature = "serial")]
pub mod usb;

use std::path::PathBuf;

pub use any_machine::{AnyMachine, AnyMachineInfo};
pub use discover::Discover;
pub use file::TemporaryFile;
pub use machine::Machine;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub use slicer::AnySlicer;
pub use sync::SharedMachine;
pub use traits::{
    Control, FdmHardwareConfiguration, FilamentMaterial, GcodeControl, GcodeSlicer, GcodeTemporaryFile,
    HardwareConfiguration, MachineInfo, MachineMakeModel, MachineState, MachineType, SlicerConfiguration,
    SuspendControl, TemperatureSensor, TemperatureSensorReading, TemperatureSensors, ThreeMfControl, ThreeMfSlicer,
    ThreeMfTemporaryFile,
};

/// A specific file containing a design to be manufactured.
#[non_exhaustive]
pub enum DesignFile {
    /// Stl ("stereolithography") 3D export, as seen in `.stl` (`model/stl`)
    /// files.
    Stl(PathBuf),
}

/// Set of three values to represent the extent of a 3-D Volume. This contains
/// the width, depth, and height values, generally used to represent some
/// maximum or minimum.
///
/// All measurements are in millimeters.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Volume {
    /// Width of the volume ("left and right"), in millimeters.
    pub width: f64,

    /// Depth of the volume ("front to back"), in millimeters.
    pub depth: f64,

    /// Height of the volume ("up and down"), in millimeters.
    pub height: f64,
}
