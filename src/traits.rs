use crate::{DesignFile, TemporaryFile, Volume};
use std::future::Future;

/// Specific technique by which this Machine takes a design, and produces
/// a real-world 3D object.
#[derive(Debug, Copy, Clone)]
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
#[derive(Debug, Clone)]
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
    /// Error type returned by this trait, and any relient traits.
    type Error;

    /// Handle to control the Machine.
    type Control: Control;

    /// Return the mechanism by which this machine will take a design and
    /// produce a real-world object.
    fn machine_type(&self) -> MachineType;

    /// Return the make/model/serial number of the reachable
    /// Machine.
    fn make_model(&self) -> MachineMakeModel;

    /// Return a handle to the Control channel of the discovered machine.
    fn control(&self) -> impl Future<Output = Result<Self::Control, Self::Error>>;
}

/// Trait implemented by schemes that can dynamically resolve Machines that can
/// be controlled by the `machine-api`.
pub trait Discover {
    /// Error type returned by this trait, and any relient traits.
    type Error;

    /// Underlying type containing information about the discovered printer.
    type MachineInfo: MachineInfo;

    /// Discover all printers on the network.
    ///
    /// This will continuously search for printers until the program is
    /// stopped. You likely want to spawn this on a separate tokio task.
    fn discover(&self) -> impl Future<Output = Result<(), Self::Error>>;

    /// Return all discovered printers.
    fn discovered(&self) -> impl Future<Output = Result<Vec<Self::MachineInfo>, Self::Error>>;
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

/// [ControlGcode] is used by Machines that accept gcode, control commands
/// that are produced from a slicer from a design file.
pub trait ControlGcode
where
    Self: Control,
{
    /// Build a 3D object from the provided *gcode* file. The generated gcode
    /// must be generated for the specific machine, and machine configuration.
    fn build(&self, job_name: &str, gcode: TemporaryFile) -> impl Future<Output = Result<(), Self::Error>>;
}

/// [ControlSuspend] is used by [Control] handles that can pause
/// and resume the current job.
pub trait ControlSuspend
where
    Self: Control,
{
    /// Request that the Machine pause manufacturing the current part,
    /// which may be resumed later.
    fn pause(&self) -> impl Future<Output = Result<(), Self::Error>>;

    /// Request that the Machine resume manufacturing the paused job to
    /// manufacturer a part.
    fn resume(&self) -> impl Future<Output = Result<(), Self::Error>>;
}

/// [Control]-specific slicer which takes a particular [DesignFile], and produces
/// GCode.
pub trait Slicer {
    /// Error type returned by this trait.
    type Error;

    /// Take an input design file, and return a handle to a File on the
    /// filesystem which contains the gcode to be sent to the Machine.
    fn generate(
        &self,
        design_file: &DesignFile,
    ) -> impl Future<Output = Result<TemporaryFile, <Self as Slicer>::Error>>;
}
