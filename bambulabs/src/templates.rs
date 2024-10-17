//! Templates for the machine, filament, and process settings.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The template for the machine, filament, and process settings.
/// For the slicer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Template {
    /// The template for the machine settings.
    Machine(Box<Machine>),
    /// The template for the filament settings.
    Filament(Filament),
    /// The template for the process settings.
    Process(Process),
}

/// The machine settings template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Machine {
    /// The name of the machine.
    pub name: String,
    /// The inheritance information of the machine.
    pub inherits: String,
    /// The origin or source of the machine.
    pub from: String,
    /// The unique identifier for the machine's settings.
    pub setting_id: String,
    /// The instantiation details of the machine.
    pub instantiation: String,
    /// A list of nozzle diameters supported by the machine.
    pub nozzle_diameter: Vec<String>,
    /// The current bed type of the machine.
    pub curr_bed_type: String,
    /// The model of the printer.
    pub printer_model: String,
    /// The variant of the printer.
    pub printer_variant: String,
    /// Areas of the bed to exclude from printing.
    pub bed_exclude_area: Vec<String>,
    /// Default filament profiles for the machine.
    pub default_filament_profile: Vec<String>,
    /// Default print profile for the machine.
    pub default_print_profile: String,
    /// Offset values for the extruder.
    pub extruder_offset: Vec<String>,
    /// Time taken to load filament in the machine.
    pub machine_load_filament_time: String,
    /// Time taken to unload filament from the machine.
    pub machine_unload_filament_time: String,
    /// Whether to scan the first layer.
    pub scan_first_layer: String,
    /// List of machines that are upward compatible.
    pub upward_compatible_machine: Vec<String>,
    /// G-code to run at the start of a print job.
    pub machine_start_gcode: String,
    /// G-code to run when changing filament.
    pub change_filament_gcode: String,
    /// The printable area dimensions.
    pub printable_area: Vec<String>,
    /// Information about the auxiliary fan.
    pub auxiliary_fan: String,
    /// Colors associated with each extruder.
    pub extruder_colour: Vec<String>,
    /// Maximum acceleration for the extruder.
    pub machine_max_acceleration_e: Vec<String>,
    /// Maximum acceleration while extruding.
    pub machine_max_acceleration_extruding: Vec<String>,
    /// Maximum acceleration while retracting.
    pub machine_max_acceleration_retracting: Vec<String>,
    /// Maximum acceleration during travel moves.
    pub machine_max_acceleration_travel: Vec<String>,
    /// Maximum acceleration for X-axis.
    pub machine_max_acceleration_x: Vec<String>,
    /// Maximum acceleration for Y-axis.
    pub machine_max_acceleration_y: Vec<String>,
    /// Maximum acceleration for Z-axis.
    pub machine_max_acceleration_z: Vec<String>,
    /// Maximum speed for the extruder.
    pub machine_max_speed_e: Vec<String>,
    /// Maximum speed for X-axis.
    pub machine_max_speed_x: Vec<String>,
    /// Maximum speed for Y-axis.
    pub machine_max_speed_y: Vec<String>,
    /// Maximum speed for Z-axis.
    pub machine_max_speed_z: Vec<String>,
    /// Maximum jerk for the extruder.
    pub machine_max_jerk_e: Vec<String>,
    /// Maximum jerk for X-axis.
    pub machine_max_jerk_x: Vec<String>,
    /// Maximum jerk for Y-axis.
    pub machine_max_jerk_y: Vec<String>,
    /// Maximum jerk for Z-axis.
    pub machine_max_jerk_z: Vec<String>,
    /// Minimum rate for extruding.
    pub machine_min_extruding_rate: Vec<String>,
    /// Minimum rate for travel moves.
    pub machine_min_travel_rate: Vec<String>,
    /// Z-lift values for retractions below certain heights.
    pub retract_lift_below: Vec<String>,
    /// Clearance radius around the extruder.
    pub extruder_clearance_radius: String,
    /// Maximum clearance radius around the extruder.
    pub extruder_clearance_max_radius: String,
    /// Clearance height from extruder to printer lid.
    pub extruder_clearance_height_to_lid: String,
    /// Volume of the nozzle.
    pub nozzle_volume: String,
    /// Structure type of the printer.
    pub printer_structure: String,
    /// Best position for placing objects on the print bed.
    pub best_object_pos: String,
    /// Minimum travel distance for retraction.
    pub retraction_minimum_travel: Vec<String>,
    /// Whether to retract before wiping.
    pub retract_before_wipe: Vec<String>,
    /// Length of filament to retract.
    pub retraction_length: Vec<String>,
    /// Length of filament to retract when changing tools.
    pub retract_length_toolchange: Vec<String>,
    /// Z-hop settings for retractions.
    pub z_hop: Vec<String>,
    /// Speed of retraction.
    pub retraction_speed: Vec<String>,
    /// Speed of de-retraction (returning filament after retraction).
    pub deretraction_speed: Vec<String>,
    /// Types of Z-hop movements supported.
    pub z_hop_types: Vec<String>,
    /// Type of nozzle used in the printer.
    pub nozzle_type: String,
    /// Whether the printer supports single extruder multi-material printing.
    pub single_extruder_multi_material: String,
    /// G-code to run at the end of a print job.
    pub machine_end_gcode: String,
    /// G-code to run when changing layers.
    pub layer_change_gcode: String,
    /// G-code to run when pausing the print.
    pub machine_pause_gcode: String,
    /// Whether the printer supports chamber temperature control.
    pub support_chamber_temp_control: String,
    /// The technology used by the printer (e.g., FDM, SLA).
    pub printer_technology: String,
    /// The flavor of G-code used by the printer.
    pub gcode_flavor: String,
    /// Whether the printer has a silent mode.
    pub silent_mode: String,
    /// Maximum layer height supported by the printer.
    pub max_layer_height: Vec<String>,
    /// Minimum layer height supported by the printer.
    pub min_layer_height: Vec<String>,
    /// Maximum printable height of the printer.
    pub printable_height: String,
    /// Clearance height from extruder to printer rod.
    pub extruder_clearance_height_to_rod: String,
    /// Identifier for the printer settings.
    pub printer_settings_id: String,
    /// Whether to retract when changing layers.
    pub retract_when_changing_layer: Vec<String>,
    /// Extra length to extrude after retraction.
    pub retract_restart_extra: Vec<String>,
    /// Extra length to extrude after retraction when changing tools.
    pub retract_restart_extra_toolchange: Vec<String>,
    /// Whether the printer supports air filtration.
    pub support_air_filtration: String,
    /// Wipe settings for the nozzle.
    pub wipe: Vec<String>,
}

/// The filament settings template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Filament {
    /// The name of the filament.
    pub name: String,
    /// The inheritance information of the filament.
    pub inherits: String,
    /// The origin or source of the filament.
    pub from: String,
    /// The unique identifier for the filament's settings.
    pub setting_id: String,
    /// The instantiation details of the filament.
    pub instantiation: String,
    /// Maximum speed settings for the cooling fan.
    pub fan_max_speed: Vec<String>,
    /// Maximum volumetric speed for the filament.
    pub filament_max_volumetric_speed: Vec<String>,
    /// Temperature settings for the nozzle when using this filament.
    pub nozzle_temperature: Vec<String>,
    /// Minimum layer time for slowing down print speed.
    pub slow_down_layer_time: Vec<String>,
    /// Minimum speed when slowing down for cooling.
    pub slow_down_min_speed: Vec<String>,
    /// List of printers compatible with this filament.
    pub compatible_printers: Vec<String>,
}

/// The process settings template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Process {
    /// The name of the process.
    pub name: String,
    /// The inheritance information of the process.
    pub inherits: String,
    /// The origin or source of the process.
    pub from: String,
    /// The unique identifier for the process settings.
    pub setting_id: String,
    /// The instantiation details of the process.
    pub instantiation: String,
    /// A description of the process.
    pub description: String,
    /// The coefficient for smoothing operations in the process.
    pub smooth_coefficient: String,
    /// The speed setting for printing totally overhanging parts.
    pub overhang_totally_speed: String,
    /// List of printers compatible with this process.
    pub compatible_printers: Vec<String>,
}
