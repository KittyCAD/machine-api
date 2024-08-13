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
mod file;
#[cfg(feature = "formlabs")]
pub mod formlabs;
pub mod gcode;
#[cfg(feature = "moonraker")]
pub mod moonraker;
pub mod server;
pub mod slicer;
mod traits;

pub use any_machine::{AnyMachine, AnyMachineInfo};
pub use file::TemporaryFile;
pub use traits::{Control, ControlGcode, ControlSuspend, Discover, MachineInfo, MachineMakeModel, MachineType, Slicer};

use std::path::PathBuf;

/// A specific file containing a design to be manufactured.
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
#[derive(Debug, Copy, Clone)]
pub struct Volume {
    /// Width of the volume ("left and right"), in millimeters.
    pub width: f64,

    /// Depth of the volume ("front to back"), in millimeters.
    pub depth: f64,

    /// Height of the volume ("up and down"), in millimeters.
    pub height: f64,
}
