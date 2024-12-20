use std::{collections::HashMap, future::Future};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{DesignFile, TemporaryFile, Volume};

/// Specific technique by which this Machine takes a design, and produces
/// a real-world 3D object.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
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

/// Current state of the machine -- be it printing, idle or offline. This can
/// be used to determine if a printer is in the correct state to take a new
/// job.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum MachineState {
    /// If a print state can not be resolved at this time, an Unknown may
    /// be returned.
    Unknown,

    /// Idle, and ready for another job.
    Idle,

    /// Running a job -- 3D printing or CNC-ing a part.
    Running,

    /// Machine is currently offline or unreachable.
    Offline,

    /// Job is underway but halted, waiting for some action to take place.
    Paused,

    /// Job is finished, but waiting manual action to move back to Idle.
    Complete,

    /// The printer has failed and is in an unknown state that may require
    /// manual attention to resolve. The inner value is a human
    /// readable description of what specifically has failed.
    Failed {
        /// A human-readable message describing the failure.
        message: Option<String>,
    },
}

/// The material that the filament is made of.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Copy)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum FilamentMaterial {
    /// Polylactic acid based plastics
    #[default]
    Pla,

    /// Pla support
    PlaSupport,

    /// acrylonitrile butadiene styrene based plastics
    Abs,

    /// polyethylene terephthalate glycol based plastics
    Petg,

    /// unsuprisingly, nylon based
    Nylon,

    /// thermoplastic polyurethane based urethane material
    Tpu,

    /// polyvinyl alcohol based material
    Pva,

    /// high impact polystyrene based material
    Hips,

    /// composite material with stuff in other stuff, something like
    /// PLA mixed with carbon fiber, kevlar, or fiberglass
    Composite,

    /// Unknown material
    Unknown,
}

/// Information about the filament being used in a FDM printer.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Filament {
    /// The name of the filament, this is likely specfic to the manufacturer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The material that the filament is made of.
    pub material: FilamentMaterial,
    /// The color (as hex without the `#`) of the filament, this is likely specific to the manufacturer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(max = 6, min = 6))]
    pub color: Option<String>,
}

/// Configuration for a FDM-based printer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FdmHardwareConfiguration {
    /// Diameter of the extrusion nozzle, in mm.
    pub nozzle_diameter: f64,

    /// The filaments the printer has access to.
    pub filaments: Vec<Filament>,

    /// The currently loaded filament index.
    pub loaded_filament_idx: Option<usize>,
}

/// The hardware configuration of a machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum HardwareConfiguration {
    /// No configuration is possible. This isn't the same conceptually as
    /// an `Option<HardwareConfiguration>`, because this indicates we positively
    /// know there is no possible configuration changes that are possible
    /// with this method of manufcture.
    None,

    /// Hardware configuration specific to FDM based printers
    Fdm {
        /// The configuration for the FDM printer.
        config: FdmHardwareConfiguration,
    },
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

    /// Return the state of the printer.
    fn state(&self) -> impl Future<Output = Result<MachineState, Self::Error>>;

    /// Return the percentage of completion of the job. This may return an
    /// error when connecting to the machine, and it may return None if
    /// there is no job running, or if there's no way to know the progress.
    fn progress(&self) -> impl Future<Output = Result<Option<f64>, Self::Error>>;

    // TODO: look at merging MachineType and HardwareConfiguration; they
    // communicate VERY similar things conceptually.

    /// Return information about the user-controllable hardware configuration
    /// of the machine.
    fn hardware_configuration(&self) -> impl Future<Output = Result<HardwareConfiguration, Self::Error>>;
}

/// [TemperatureSensor] indicates the specific part of the machine that the
/// sensor is attached to.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TemperatureSensor {
    /// This sensor measures the temperature of the extruder of a
    /// FDM printer.
    Extruder,

    /// This sensor measures the temperature of the print bed.
    Bed,

    /// This sensor measures the temperature of a 3d print chamber.
    Chamber,
}

/// Temperature read from a sensor *ALWAYS IN CELSIUS*!
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TemperatureSensorReading {
    /// The specific temperature value observed on or near the machine.
    pub temperature_celsius: f64,

    /// If set, the desired temperature that the machine will attempt to
    /// stabalize to.
    pub target_temperature_celsius: Option<f64>,
}

/// The [TemperatureSensors] trait is implemented on Machines that are capable
/// of reporting thermal state to the caller.
pub trait TemperatureSensors {
    /// Error type returned by this trait.
    type Error;

    /// List all attached Sensors. This must not change during runtime.
    fn sensors(&self) -> impl Future<Output = Result<HashMap<String, TemperatureSensor>, Self::Error>> + Send;

    /// Poll all sensors returned by [TemperatureSensors::sensors].
    fn poll_sensors(
        &mut self,
    ) -> impl Future<Output = Result<HashMap<String, TemperatureSensorReading>, Self::Error>> + Send;
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

/// The slicer configuration is a set of parameters that are passed to the
/// slicer to control how the gcode is generated.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Copy)]
pub struct SlicerConfiguration {
    /// The filament to use for the print.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filament_idx: Option<usize>,
}

/// Options passed along with the Build request that are specific to a
/// (Machine, DesignFile and Slicer).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BuildOptions {
    /// Specific configuration of the hardware platform producing the file
    /// in real life.
    pub hardware_configuration: HardwareConfiguration,

    /// Requested configuration of the slicer -- things like supports,
    /// infill or other configuration to the designed file.
    pub slicer_configuration: SlicerConfiguration,

    /// Make/Model of the target platform.
    pub make_model: MachineMakeModel,

    /// Method by which the machine can create a physical 3D part.
    pub machine_type: MachineType,

    /// Largest build volume that the machine can construct.
    pub max_part_volume: Option<Volume>,
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
        options: &BuildOptions,
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
        options: &BuildOptions,
    ) -> impl Future<Output = Result<ThreeMfTemporaryFile, <Self as ThreeMfSlicer>::Error>>;
}

/// ThreeMfTemporaryFile is a TemporaryFile full of .3mf.
pub struct ThreeMfTemporaryFile(pub TemporaryFile);
