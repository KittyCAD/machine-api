//! Common traits used throughout this crate to manage the creation of
//! physical 3D objects.

use std::{error::Error, future::Future, path::PathBuf};
use tokio::io::AsyncRead;

/// A specific file containing a design to be manufactured.
pub enum DesignFile {
    /// Stl ("stereolithography") 3D export, as seen in `.stl` (`model/stl`)
    /// files.
    Stl(PathBuf),
}

/// Set of three values to represent the extent of a 3-D Volume. This contains
/// the width, depth, and height values, generally used to represent some
/// maximum or minimum.
#[derive(Debug, Copy, Clone)]
pub struct Volume {
    /// Width of the volume ("left and right").
    pub width: f64,

    /// Depth of the volume ("front to back").
    pub depth: f64,

    /// Height of the volume ("up and down").
    pub height: f64,
}

/// A `Machine` is something that can take a 3D model (in one of the
/// supported formats), and create a physical, real-world copy of
/// that model.
///
/// Some examples of what this crate calls a "Machine" are 3D printers,
/// CNC machines, or a service that takes a drawing and mails you back
/// a part.
pub trait MachineControl {
    /// Error type returned by this trait, and any relient traits.
    type Error: Error;

    /// Return the maximum part volume. For a 3D printer this is the bed's
    /// dimension, for a CNC, this would be the bed where the material is placed.
    fn max_part_volume(&self) -> impl Future<Output = Result<Volume, Self::Error>>;

    /// Request an immediate and complete shutdown of the equipment,
    /// requiring human intervention to bring the machine back online.
    ///
    /// This is *not* an estop as defined by things like IEC 60204-1,
    /// but it's as close as we can get over the network. This request may
    /// be enqueued and other operations may take place before the shutdown
    /// request is processed. This is not a substitute for a real physical
    /// estop -- but it's better than nothing.
    fn emergency_stop(&self) -> impl Future<Output = Result<(), Self::Error>>;

    /// Request that the machine stop any current job(s).
    fn stop(&self) -> impl Future<Output = Result<(), Self::Error>>;
}

/// [MachineControlGcode] is used by [Machine]s that accept gcode, control commands
/// that are produced from a slicer from a design file.
pub trait MachineControlGcode
where
    Self: MachineControl,
{
    /// Build a 3D object from the provided *gcode* file. The generated gcode
    /// must be generated for the specific machine, and machine configuration.
    fn build(&self, job_name: &str, gcode: impl AsyncRead) -> impl Future<Output = Result<(), Self::Error>>;
}

/// [MachineControlSuspend] is used by [MachineControl] handles that can pause
/// and resume the current job.
pub trait MachineControlSuspend
where
    Self: MachineControl,
{
    /// Request that the [Machine] pause manufacturing the current part,
    /// which may be resumed later.
    fn pause(&self) -> impl Future<Output = Result<(), Self::Error>>;

    /// Request that the [Machine] resume manufacturing the paused job to
    /// manufacturer a part.
    fn resume(&self) -> impl Future<Output = Result<(), Self::Error>>;
}

/// [MachineControl]-specific slicer which takes a particular [DesignFile], and produces
/// GCode.
pub trait Slicer {
    /// Error type returned by this trait.
    type Error: Error;

    /// Take an input design file, and return a handle to an [AsyncRead]
    /// traited object which contains the gcode to be sent to the [Machine].
    fn generate(
        &self,
        design_file: &DesignFile,
    ) -> impl Future<Output = Result<impl AsyncRead, <Self as Slicer>::Error>>;
}