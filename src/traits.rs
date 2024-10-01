use std::future::Future;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{DesignFile, TemporaryFile, Volume};

/// Specific technique by which this Machine takes a design, and produces
/// a real-world 3D object.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum MachineType {
    /// Use light to cure a resin to build up layers.
    Stereolithography,

    /// Fused Deposition Modeling, layers of melted plastic.
    FusedDeposition,

    /// "Computer numerical control" - machine that grinds away material
    /// from a hunk of material to construct a part.
    Cnc,
}

/// Information regarding the make/model of a discovered endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MachineMakeModel {
    /// The manufacturer that built the connected Machine.
    pub manufacturer: Option<String>,

    /// The model of the connected Machine.
    pub model: Option<String>,

    /// The unique serial number of the connected Machine.
    pub serial: Option<String>,
}

/// Metadata about a Machine.
pub trait MachineInfo {
    /// Return the mechanism by which this machine will take a design and
    /// produce a real-world object.
    fn machine_type(&self) -> MachineType;

    /// Return the make/model/serial number of the reachable
    /// Machine.
    fn make_model(&self) -> MachineMakeModel;

    /// Return the maximum part volume. For a 3D printer this is the bed's
    /// dimension, for a CNC, this would be the bed where the material is placed.
    ///
    /// If the part volume is not known, a None may be used.
    fn max_part_volume(&self) -> Option<Volume>;
}

/// A `Machine` is something that can take a 3D model (in one of the
/// supported formats), and create a physical, real-world copy of
/// that model.
///
/// Some examples of what this crate calls a "Machine" are 3D printers,
/// CNC machines, or a service that takes a drawing and mails you back
/// a part.
pub trait Control {
    /// Error type returned by this trait, and any relient traits.
    type Error;

    /// Type that implements MachineInfo for this hardware.
    type MachineInfo: MachineInfo;

    /// Return the information about this machine.
    fn machine_info(&self) -> impl Future<Output = Result<Self::MachineInfo, Self::Error>>;

    /// Request an immediate and complete shutdown of the equipment,
    /// requiring human intervention to bring the machine back online.
    ///
    /// This is *not* an estop as defined by things like IEC 60204-1,
    /// but it's as close as we can get over the network. This request may
    /// be enqueued and other operations may take place before the shutdown
    /// request is processed. This is not a substitute for a real physical
    /// estop -- but it's better than nothing.
    fn emergency_stop(&mut self) -> impl Future<Output = Result<(), Self::Error>>;

    /// Request that the machine stop any current job(s).
    fn stop(&mut self) -> impl Future<Output = Result<(), Self::Error>>;

    /// Check to see if the machine is still connected and this connection
    /// remains alive.
    ///
    /// `true` means the machine is alive and healthy, ready for use.
    /// `false` means the machine is no longer reachable or usable, and
    /// ought to be removed.
    fn healthy(&self) -> impl Future<Output = bool>;
}

/// [ControlGcode] is used by Machines that accept gcode, control commands
/// that are produced from a slicer from a design file.
pub trait GcodeControl
where
    Self: Control,
{
    /// Build a 3D object from the provided *gcode* file. The generated gcode
    /// must be generated for the specific machine, and machine configuration.
    fn build(&mut self, job_name: &str, gcode: GcodeTemporaryFile) -> impl Future<Output = Result<(), Self::Error>>;
}

/// [Control3Mf] is used by Machines that accept .3mf, control commands
/// that are produced from a slicer from a design file.
pub trait ThreeMfControl
where
    Self: Control,
{
    /// Build a 3D object from the provided *.3mf* file. The generated 3mf
    /// must be generated for the specific machine, and machine configuration.
    fn build(
        &mut self,
        job_name: &str,
        three_mf: ThreeMfTemporaryFile,
    ) -> impl Future<Output = Result<(), Self::Error>>;
}

/// [ControlSuspend] is used by [Control] handles that can pause
/// and resume the current job.
pub trait SuspendControl
where
    Self: Control,
{
    /// Request that the Machine pause manufacturing the current part,
    /// which may be resumed later.
    fn pause(&mut self) -> impl Future<Output = Result<(), Self::Error>>;

    /// Request that the Machine resume manufacturing the paused job to
    /// manufacturer a part.
    fn resume(&mut self) -> impl Future<Output = Result<(), Self::Error>>;
}

/// [Control]-specific slicer which takes a particular [DesignFile], and produces
/// GCode.
pub trait GcodeSlicer {
    /// Error type returned by this trait.
    type Error;

    /// Take an input design file, and return a handle to a File on the
    /// filesystem which contains the gcode to be sent to the Machine.
    fn generate(
        &self,
        design_file: &DesignFile,
    ) -> impl Future<Output = Result<GcodeTemporaryFile, <Self as GcodeSlicer>::Error>>;
}

/// GcodeTemporaryFile is a TemporaryFile full of .gcode.
pub struct GcodeTemporaryFile(pub TemporaryFile);

/// [Control]-specific slicer which takes a particular [DesignFile], and produces
/// GCode.
pub trait ThreeMfSlicer {
    /// Error type returned by this trait.
    type Error;

    /// Take an input design file, and return a handle to a File on the
    /// filesystem which contains the .3mf to be sent to the Machine.
    fn generate(
        &self,
        design_file: &DesignFile,
    ) -> impl Future<Output = Result<ThreeMfTemporaryFile, <Self as ThreeMfSlicer>::Error>>;
}

/// ThreeMfTemporaryFile is a TemporaryFile full of .3mf.
pub struct ThreeMfTemporaryFile(pub TemporaryFile);
