//! Common traits used throughout this crate to manage the creation of
//! physical 3D objects.

use std::{error::Error, path::Path};
use tokio::io::AsyncRead;

/// A `Machine` is something that can take a 3D model (in one of the
/// supported formats), and create a physical, real-world copy of
/// that model.
///
/// Some examples of what this crate calls a "Machine" are 3D printers,
/// CNC machines, or a service that takes a drawing and mails you back
/// a part.
pub trait Machine {
    /// Error type returned by this trait.
    type Error: Error;

    /// Request an immediate and complete shutdown of the equipment,
    /// requiring human intervention to bring the machine back online.
    ///
    /// This is *not* an estop as defined by things like IEC 60204-1,
    /// but it's as close as we can get over the network. This request may
    /// be enqueued and other operations may take place before the shutdown
    /// request is processed. This is not a substitute for a real physical
    /// estop -- but it's better than nothing.
    async fn emergency_stop(&self) -> Result<(), Self::Error>;

    /// Request that the machine stop any current job(s).
    async fn stop(&self) -> Result<(), Self::Error>;
}

/// [Machine]-specific slicer which takes a particular DesignFile, and produces
/// GCode.
pub trait MachineSlicer {
    /// Error type returned by this trait.
    type Error: Error;

    /// Take an input design file, and return a handle to an [AsyncRead]
    /// traited object which contains the gcode to be sent to the [Machine].
    async fn generate(&self, design_file: &Path) -> Result<impl AsyncRead, Self::Error>;
}

/// GcodeMachine is used by [Machine]s that accept gcode, control commands
/// that are produced from a slicer from a design file.
pub trait GcodeMachine {
    /// Error type returned by this trait.
    type Error: Error;

    /// Build a 3D object from the provided *gcode* file. The generated gcode
    /// must be generated for the specific machine, and machine configuration.
    async fn build(&self, job_name: &str, file: &Path) -> Result<(), Self::Error>;
}

/// SuspendMachine is used by [Machine]s that can pause and resume the current
/// job.
pub trait SuspendMachine {
    /// Error type returned by this trait.
    type Error: Error;

    /// Request that the [Machine] pause manufacturing the current part,
    /// which may be resumed later.
    async fn pause(&self) -> Result<(), Self::Error>;

    /// Request that the [Machine] resume manufacturing the paused job to
    /// manufacturer a part.
    async fn resume(&self) -> Result<(), Self::Error>;
}
