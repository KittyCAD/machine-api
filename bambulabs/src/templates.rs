//! Templates for the machine, filament, and process settings.

use std::collections::BTreeMap;

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::message::NozzleDiameter;

/// Template directory.
static TEMPLATE_DIR: include_dir::Dir<'_> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/profiles");

/// The template for the machine, filament, and process settings.
/// For the slicer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Template {
    /// The template for the machine settings.
    Machine(Box<Machine>),
    /// The template for a machine model.
    MachineModel(MachineModel),
    /// The template for the filament settings.
    Filament(Filament),
    /// The template for the process settings.
    Process(Process),
}

impl Template {
    /// Get the name of the template.
    pub fn name(&self) -> String {
        match self {
            Template::Machine(machine) => machine.name.clone(),
            Template::MachineModel(model) => model.name.clone(),
            Template::Filament(filament) => filament.name.clone(),
            Template::Process(process) => process.name.clone(),
        }
    }

    /// Get the other settings for the template.
    pub fn other(&self) -> BTreeMap<String, Value> {
        match self {
            Template::Machine(machine) => machine.other.clone(),
            Template::MachineModel(model) => model.other.clone(),
            Template::Filament(filament) => filament.other.clone(),
            Template::Process(process) => process.other.clone(),
        }
    }

    /// Get the inheritance information for the template.
    pub fn inherits(&self) -> Option<String> {
        match self {
            Template::Machine(machine) => machine.inherits.clone(),
            Template::MachineModel(_) => None,
            Template::Filament(filament) => filament.inherits.clone(),
            Template::Process(process) => process.inherits.clone(),
        }
    }

    /// Set inheritance information for the template.
    pub fn set_inherits(&mut self, inherits: &str) {
        match self {
            Template::Machine(machine) => {
                machine.inherits = Some(inherits.to_string());
                machine.name = inherits.to_string();
            }
            Template::MachineModel(_) => {}
            Template::Filament(filament) => {
                filament.inherits = Some(inherits.to_string());
                filament.name = inherits.to_string();
            }
            Template::Process(process) => {
                process.inherits = Some(inherits.to_string());
                process.name = inherits.to_string();
            }
        }
    }

    /// Load inherited settings from the given templates.
    /// We use serde_json::Value to merge the settings because it's easier to work with.
    /// And more generic.
    pub fn load_inherited(&self) -> Result<Template> {
        let Some(inherits) = self.inherits() else {
            // We have no inherited settings.
            return Ok(self.clone());
        };

        let glob = match self {
            Template::Machine(_) => "**/machine/*.json",
            Template::MachineModel(_) => "**/machine/*.json",
            Template::Filament(_) => "**/filament/*.json",
            Template::Process(_) => "**/process/*.json",
        };

        let mut templates = BTreeMap::new();
        for entry in TEMPLATE_DIR.find(glob)? {
            let Some(entry) = entry.as_file() else {
                continue;
            };
            let template = entry
                .contents_utf8()
                .ok_or_else(|| anyhow::anyhow!("Failed to read template file '{}'", entry.path().display()))?;
            let template: Template = serde_json::from_str(template)?;
            templates.insert(template.name(), template);
        }

        // Get the inherited template.
        let inherited = templates
            .get(&inherits)
            .ok_or_else(|| anyhow::anyhow!("Inherited template '{}' not found", inherits))?;

        let mut highest_weight = serde_json::to_value(self)?;
        let inherited = serde_json::to_value(inherited)?;

        // Merge the inherited settings into the current settings.
        if let Value::Object(highest_weight) = &mut highest_weight {
            if let Value::Object(inherited) = &inherited {
                for (key, value) in inherited {
                    if !highest_weight.contains_key(key) || key == "inherits" {
                        highest_weight.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        let new: Template = serde_json::from_value(highest_weight)?;

        if new.inherits() == self.inherits() {
            // prevent infinite loop.
            return Ok(new);
        }

        // Get the inherited settings.
        new.load_inherited()
    }
}

/// Nozzle diameter group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum NozzleDiameterGroup {
    /// A single nozzle diameter. Or semi-colon separated list of nozzle diameters.
    Single(String),
    /// A range of nozzle diameters.
    Range(Vec<NozzleDiameter>),
}

impl NozzleDiameterGroup {
    /// Get the nozzle diameter group as a vector of nozzle diameters.
    pub fn as_vec(&self) -> Result<Vec<NozzleDiameter>> {
        match self {
            NozzleDiameterGroup::Single(diameter) => {
                let diameters = diameter
                    .split(';')
                    .map(|diameter| diameter.parse::<NozzleDiameter>())
                    .collect::<Result<Vec<NozzleDiameter>, parse_display::ParseError>>()?;
                Ok(diameters)
            }
            NozzleDiameterGroup::Range(diameters) => Ok(diameters.clone()),
        }
    }
}

/// The machine settings template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Machine {
    /// The name of the machine.
    pub name: String,
    /// The inheritance information of the machine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherits: Option<String>,
    /// The origin or source of the machine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// The unique identifier for the machine's settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setting_id: Option<String>,
    /// The instantiation details of the machine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instantiation: Option<String>,
    /// A list of nozzle diameters supported by the machine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nozzle_diameter: Option<NozzleDiameterGroup>,
    /// The nozzle height of the machine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nozzle_height: Option<String>,
    /// Enable long retraction when cut.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_long_retraction_when_cut: Option<String>,
    /// Enable long retractions when cut.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enable_long_retractions_when_cut: Vec<String>,
    /// Long retractions when cut.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub long_retractions_when_cut: Vec<String>,
    /// Enable filament ramming.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_filament_ramming: Option<String>,
    /// Purge in prime tower.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purge_in_prime_tower: Option<String>,
    /// Retraction distances when cut.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retraction_distances_when_cut: Vec<String>,
    /// Head wrap detect zone.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub head_wrap_detect_zone: Vec<String>,
    /// Timelapse G-code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_lapse_gcode: Option<String>,
    /// The current bed type of the machine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub curr_bed_type: Option<String>,
    /// The model of the printer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub printer_model: Option<String>,
    /// The variant of the printer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub printer_variant: Option<String>,
    /// Areas of the bed to exclude from printing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bed_exclude_area: Vec<String>,
    /// Default filament profiles for the machine.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_filament_profile: Vec<String>,
    /// Default print profile for the machine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_print_profile: Option<String>,
    /// Offset values for the extruder.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extruder_offset: Vec<String>,
    /// Time taken to load filament in the machine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine_load_filament_time: Option<String>,
    /// Time taken to unload filament from the machine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine_unload_filament_time: Option<String>,
    /// Whether to scan the first layer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scan_first_layer: Option<String>,
    /// List of machines that are upward compatible.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upward_compatible_machine: Vec<String>,
    /// G-code to run at the start of a print job.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine_start_gcode: Option<String>,
    /// G-code to run when changing filament.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_filament_gcode: Option<String>,
    /// The printable area dimensions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub printable_area: Vec<String>,
    /// Information about the auxiliary fan.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auxiliary_fan: Option<String>,
    /// Colors associated with each extruder.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extruder_colour: Vec<String>,
    /// Maximum acceleration for the extruder.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machine_max_acceleration_e: Vec<String>,
    /// Maximum acceleration while extruding.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machine_max_acceleration_extruding: Vec<String>,
    /// Maximum acceleration while retracting.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machine_max_acceleration_retracting: Vec<String>,
    /// Maximum acceleration during travel moves.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machine_max_acceleration_travel: Vec<String>,
    /// Maximum acceleration for X-axis.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machine_max_acceleration_x: Vec<String>,
    /// Maximum acceleration for Y-axis.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machine_max_acceleration_y: Vec<String>,
    /// Maximum acceleration for Z-axis.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machine_max_acceleration_z: Vec<String>,
    /// Maximum speed for the extruder.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machine_max_speed_e: Vec<String>,
    /// Maximum speed for X-axis.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machine_max_speed_x: Vec<String>,
    /// Maximum speed for Y-axis.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machine_max_speed_y: Vec<String>,
    /// Maximum speed for Z-axis.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machine_max_speed_z: Vec<String>,
    /// Maximum jerk for the extruder.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machine_max_jerk_e: Vec<String>,
    /// Maximum jerk for X-axis.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machine_max_jerk_x: Vec<String>,
    /// Maximum jerk for Y-axis.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machine_max_jerk_y: Vec<String>,
    /// Maximum jerk for Z-axis.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machine_max_jerk_z: Vec<String>,
    /// Minimum rate for extruding.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machine_min_extruding_rate: Vec<String>,
    /// Minimum rate for travel moves.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub machine_min_travel_rate: Vec<String>,
    /// Z-lift values for retractions below certain heights.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retract_lift_below: Vec<String>,
    /// Clearance radius around the extruder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extruder_clearance_radius: Option<String>,
    /// Maximum clearance radius around the extruder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extruder_clearance_max_radius: Option<String>,
    /// Clearance height from extruder to printer lid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extruder_clearance_height_to_lid: Option<String>,
    /// Volume of the nozzle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nozzle_volume: Option<String>,
    /// Structure type of the printer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub printer_structure: Option<String>,
    /// Best position for placing objects on the print bed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best_object_pos: Option<String>,
    /// Minimum travel distance for retraction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retraction_minimum_travel: Vec<String>,
    /// Whether to retract before wiping.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retract_before_wipe: Vec<String>,
    /// Length of filament to retract.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retraction_length: Vec<String>,
    /// Length of filament to retract when changing tools.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retract_length_toolchange: Vec<String>,
    /// Z-hop settings for retractions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub z_hop: Vec<String>,
    /// Speed of retraction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retraction_speed: Vec<String>,
    /// Speed of de-retraction (returning filament after retraction).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deretraction_speed: Vec<String>,
    /// Types of Z-hop movements supported.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub z_hop_types: Vec<String>,
    /// Type of nozzle used in the printer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nozzle_type: Option<String>,
    /// Whether the printer supports single extruder multi-material printing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub single_extruder_multi_material: Option<String>,
    /// G-code to run at the end of a print job.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine_end_gcode: Option<String>,
    /// G-code to run when changing layers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer_change_gcode: Option<String>,
    /// G-code to run when pausing the print.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine_pause_gcode: Option<String>,
    /// Whether the printer supports chamber temperature control.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_chamber_temp_control: Option<String>,
    /// The technology used by the printer (e.g., FDM, SLA).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub printer_technology: Option<String>,
    /// The flavor of G-code used by the printer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gcode_flavor: Option<String>,
    /// Whether the printer has a silent mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub silent_mode: Option<String>,
    /// Maximum layer height supported by the printer.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub max_layer_height: Vec<String>,
    /// Minimum layer height supported by the printer.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub min_layer_height: Vec<String>,
    /// Maximum printable height of the printer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub printable_height: Option<String>,
    /// Clearance height from extruder to printer rod.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extruder_clearance_height_to_rod: Option<String>,
    /// Identifier for the printer settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub printer_settings_id: Option<String>,
    /// Whether to retract when changing layers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retract_when_changing_layer: Vec<String>,
    /// Extra length to extrude after retraction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retract_restart_extra: Vec<String>,
    /// Extra length to extrude after retraction when changing tools.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retract_restart_extra_toolchange: Vec<String>,
    /// Whether the printer supports air filtration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_air_filtration: Option<String>,
    /// Wipe settings for the nozzle.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wipe: Vec<String>,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// The machine model template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MachineModel {
    /// The name of the machine model.
    pub name: String,
    /// The diameter of the nozzle used in this machine model.
    pub nozzle_diameter: NozzleDiameterGroup,
    /// The URL associated with this machine model, possibly for more information or resources.
    pub url: url::Url,
    /// The 3D model file for the print bed of this machine.
    pub bed_model: String,
    /// The texture file for the print bed of this machine.
    pub bed_texture: String,
    /// The default type of bed used in this machine model.
    pub default_bed_type: String,
    /// The family or series of printers this model belongs to.
    pub family: String,
    /// The technology used by this machine (e.g., FDM, SLA).
    pub machine_tech: String,
    /// A unique identifier for this machine model.
    pub model_id: String,
    /// The default materials compatible with this machine model.
    pub default_materials: String,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// The filament settings template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Filament {
    /// The name of the filament.
    pub name: String,
    /// The description of the filament.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The inheritance information of the filament.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherits: Option<String>,
    /// The origin or source of the filament.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// The unique identifier for the filament's settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setting_id: Option<String>,
    /// The instantiation details of the filament.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instantiation: Option<String>,
    /// Maximum speed settings for the cooling fan.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fan_max_speed: Vec<String>,
    /// Maximum volumetric speed for the filament.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_max_volumetric_speed: Vec<String>,
    /// Temperature settings for the nozzle when using this filament.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nozzle_temperature: Vec<String>,
    /// Minimum layer time for slowing down print speed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slow_down_layer_time: Vec<String>,
    /// Minimum speed when slowing down for cooling.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slow_down_min_speed: Vec<String>,
    /// List of printers compatible with this filament.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub compatible_printers: Vec<String>,
    /// The G-code to be executed at the start of filament use.
    /// This is an array of strings, typically containing a single multi-line G-code script.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_start_gcode: Vec<String>,
    /// The flow ratio for the filament. Typically a decimal value slightly below 1.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_flow_ratio: Vec<String>,
    /// The vendor or manufacturer of the filament.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_vendor: Vec<String>,
    /// The time (in seconds) each layer should be cooled by the fan.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fan_cooling_layer_time: Vec<String>,
    /// The minimum speed of the cooling fan.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fan_min_speed: Vec<String>,
    /// The temperature of the hot plate during printing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hot_plate_temp: Vec<String>,
    /// The temperature of the hot plate for the initial layer.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hot_plate_temp_initial_layer: Vec<String>,
    /// The nozzle temperature for the initial layer.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nozzle_temperature_initial_layer: Vec<String>,
    /// The temperature of the textured plate during printing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub textured_plate_temp: Vec<String>,
    /// The temperature of the textured plate for the initial layer.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub textured_plate_temp_initial_layer: Vec<String>,
    /// The high end of the nozzle temperature range in degrees Celsius.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nozzle_temperature_range_high: Vec<String>,
    /// Whether to reduce the frequency of fan stops and starts. 0 for disabled, 1 for enabled.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reduce_fan_stop_start_freq: Vec<String>,
    /// The cost of the filament, typically in currency units per weight or length.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_cost: Vec<String>,
    /// The unique identifier for the filament.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filament_id: Option<String>,
    /// The density of the filament in g/cm³.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_density: Vec<String>,
    /// The low end of the nozzle temperature range in degrees Celsius.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nozzle_temperature_range_low: Vec<String>,
    /// The vitrification (glass transition) temperature of the filament in degrees Celsius.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub temperature_vitrification: Vec<String>,
    /// Chamber temperatures.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chamber_temperatures: Vec<String>,
    /// Overhang fan speed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overhang_fan_speed: Vec<String>,
    /// Full fan speed layer.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub full_fan_speed_layer: Vec<String>,
    /// The additional speed for the cooling fan, as a percentage.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_cooling_fan_speed: Vec<String>,
    /// The number of initial layers to print without the cooling fan.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub close_fan_the_first_x_layers: Vec<String>,
    /// The temperature of the cool plate during printing, in degrees Celsius.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cool_plate_temp: Vec<String>,
    /// The temperature of the cool plate for the initial layer, in degrees Celsius.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cool_plate_temp_initial_layer: Vec<String>,
    /// The temperature of the engineering plate during printing, in degrees Celsius.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub eng_plate_temp: Vec<String>,
    /// The temperature of the engineering plate for the initial layer, in degrees Celsius.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub eng_plate_temp_initial_layer: Vec<String>,
    /// Whether the filament is intended for support structures (1 for yes, 0 for no).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_is_support: Vec<String>,
    /// Whether the filament is soluble (1 for yes, 0 for no).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_soluble: Vec<String>,
    /// The type of filament (e.g., PVA, PLA, ABS).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_type: Vec<String>,
    /// The overhang angle threshold for increasing fan speed, as a percentage.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overhang_fan_threshold: Vec<String>,
    /// Filament retraction length.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_retraction_length: Vec<String>,
    /// Whether to perform long retractions when the filament is cut (1 for yes, 0 for no).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_long_retractions_when_cut: Vec<String>,
    /// The distance to retract the filament when it's cut, in millimeters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_retraction_distances_when_cut: Vec<String>,
    /// Activate air filtration.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub activate_air_filtration: Vec<String>,
    /// Required nozzle HRC.
    #[serde(default, skip_serializing_if = "Vec::is_empty", alias = "required_nozzle_HRC")]
    pub required_nozzle_hrc: Vec<String>,
    /// Whether to retract the filament before wiping. 'nil' might indicate a default or unset value.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_retract_before_wipe: Vec<String>,
    /// The distance to wipe the nozzle after retraction, in millimeters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_wipe_distance: Vec<String>,
    /// The type of Z-hop movement to perform during retraction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_z_hop_types: Vec<String>,
    /// The speed of the exhaust fan after print completion, as a percentage.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub complete_print_exhaust_fan_speed: Vec<String>,
    /// The speed of the exhaust fan during printing, as a percentage.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub during_print_exhaust_fan_speed: Vec<String>,
    /// The speed of filament de-retraction. 'nil' might indicate a default or unset value.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_deretraction_speed: Vec<String>,
    /// The diameter of the filament in millimeters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_diameter: Vec<String>,
    /// G-code to be executed at the end of using this filament.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_end_gcode: Vec<String>,
    /// Minimal amount of filament to purge on the wipe tower, in millimeters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_minimal_purge_on_wipe_tower: Vec<String>,
    /// Extra length of filament to push after retraction. 'nil' might indicate a default or unset value.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_retract_restart_extra: Vec<String>,
    /// Whether to retract when changing layers. 'nil' might indicate a default or unset value.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_retract_when_changing_layer: Vec<String>,
    /// Minimum travel distance that triggers a retraction. 'nil' might indicate a default or unset value.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_retraction_minimum_travel: Vec<String>,
    /// Speed of filament retraction. 'nil' might indicate a default or unset value.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_retraction_speed: Vec<String>,
    /// Identifier for these filament settings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_settings_id: Vec<String>,
    /// Whether to perform a wipe movement after retraction. 'nil' might indicate a default or unset value.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_wipe: Vec<String>,
    /// Z-hop height for filament retraction. 'nil' might indicate a default or unset value.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filament_z_hop: Vec<String>,
    /// Whether to slow down printing for layer cooling (1 for yes, 0 for no).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slow_down_for_layer_cooling: Vec<String>,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

/// The process settings template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Process {
    /// The name of the process.
    pub name: String,
    /// The inheritance information of the process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherits: Option<String>,
    /// The origin or source of the process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// The unique identifier for the process settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setting_id: Option<String>,
    /// The instantiation details of the process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instantiation: Option<String>,
    /// A description of the process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The coefficient for smoothing operations in the process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smooth_coefficient: Option<String>,
    /// The speed setting for printing totally overhanging parts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overhang_totally_speed: Option<String>,
    /// List of printers compatible with this process.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub compatible_printers: Vec<String>,
    /// Default acceleration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_acceleration: Option<String>,
    /// Travel speed for the process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub travel_speed: Option<String>,
    /// Elefant foot compensation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elefant_foot_compensation: Option<String>,
    /// Outer wall acceleration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outer_wall_acceleration: Option<String>,
    /// Outer wall speed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outer_wall_speed: Option<String>,
    /// Sparse infill pattern.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sparse_infill_pattern: Option<String>,
    /// Initial layer infill speed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_layer_infill_speed: Option<String>,
    /// Initial layer speed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_layer_speed: Option<String>,
    /// Gap infill speed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gap_infill_speed: Option<String>,
    /// Inner wall speed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inner_wall_speed: Option<String>,
    /// Internal solid infill speed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_solid_infill_speed: Option<String>,
    /// Sparse infill speed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sparse_infill_speed: Option<String>,
    /// Top surface speed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_surface_speed: Option<String>,
    /// Bottom shell layers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bottom_shell_layers: Option<String>,
    /// Bridge flow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bridge_flow: Option<String>,
    /// Initial layer line width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_layer_line_width: Option<String>,
    /// Initial layer print height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_layer_print_height: Option<String>,
    /// Inner wall line width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inner_wall_line_width: Option<String>,
    /// Internal solid infill line width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_solid_infill_line_width: Option<String>,
    /// Layer height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer_height: Option<String>,
    /// Line width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_width: Option<String>,
    /// Outer wall line width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outer_wall_line_width: Option<String>,
    /// Sparse infill line width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sparse_infill_line_width: Option<String>,
    /// Support bottom z distance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_bottom_z_distance: Option<String>,
    /// Top shell layers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_shell_layers: Option<String>,
    /// Top surface line width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_surface_line_width: Option<String>,
    /// Wall loops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_loops: Option<String>,
    /// Support line width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_line_width: Option<String>,
    /// Support top z distance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_top_z_distance: Option<String>,
    /// Sparse infill density.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sparse_infill_density: Option<String>,
    /// Bridge speed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bridge_speed: Option<String>,
    /// Overhang 3 4 speed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overhang_3_4_speed: Option<String>,
    /// Overhang 4 4 speed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overhang_4_4_speed: Option<String>,
    /// Top surface pattern.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_surface_pattern: Option<String>,
    /// The thickness of the bottom shell layers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bottom_shell_thickness: Option<String>,
    /// The pattern used for the bottom surface layers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bottom_surface_pattern: Option<String>,
    /// The gap between the object and the brim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brim_object_gap: Option<String>,
    /// Condition for compatible printers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compatible_printers_condition: Option<String>,
    /// Whether to detect overhang walls.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detect_overhang_wall: Option<String>,
    /// The type of draft shield to use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft_shield: Option<String>,
    /// Whether to enable arc fitting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_arc_fitting: Option<String>,
    /// Whether to enable the prime tower.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_prime_tower: Option<String>,
    /// The format for the output filename.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename_format: Option<String>,
    /// The acceleration for the initial layer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initial_layer_acceleration: Option<String>,
    /// The thickness of internal bridge supports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_bridge_support_thickness: Option<String>,
    /// The flow rate for ironing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ironing_flow: Option<String>,
    /// The spacing between ironing lines.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ironing_spacing: Option<String>,
    /// The speed for ironing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ironing_speed: Option<String>,
    /// The type of ironing to perform.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ironing_type: Option<String>,
    /// The maximum distance for travel detours.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_travel_detour_distance: Option<String>,
    /// The minimum area for sparse infill.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum_sparse_infill_area: Option<String>,
    /// Whether to use only one wall for the top layer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub only_one_wall_top: Option<String>,
    /// The speed for 1/4 overhang.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overhang_1_4_speed: Option<String>,
    /// The speed for 2/4 overhang.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overhang_2_4_speed: Option<String>,
    /// The width of the prime tower.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prime_tower_width: Option<String>,
    /// Whether to reduce infill retraction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reduce_infill_retraction: Option<String>,
    /// The resolution of the print.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    /// The position of the seam.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seam_position: Option<String>,
    /// The height of the skirt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skirt_height: Option<String>,
    /// The number of skirt loops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skirt_loops: Option<String>,
    /// The spacing of the support base pattern.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_base_pattern_spacing: Option<String>,
    /// The expansion of the support.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_expansion: Option<String>,
    /// The number of bottom layers for support interface.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_interface_bottom_layers: Option<String>,
    /// The spacing for the support interface.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_interface_spacing: Option<String>,
    /// The XY distance between the object and support.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_object_xy_distance: Option<String>,
    /// The speed for printing support.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_speed: Option<String>,
    /// The style of support.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_style: Option<String>,
    /// The threshold angle for support generation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_threshold_angle: Option<String>,
    /// The type of support.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_type: Option<String>,
    /// The thickness of the top shell layers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_shell_thickness: Option<String>,
    /// The acceleration for the top surface.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_surface_acceleration: Option<String>,
    /// The angle of tree support branches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree_support_branch_angle: Option<String>,
    /// The diameter of tree support branches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree_support_branch_diameter: Option<String>,
    /// The number of walls for tree support.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree_support_wall_count: Option<String>,
    /// The type of wall generator to use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_generator: Option<String>,
    /// The order of wall and infill printing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_infill_order: Option<String>,
    /// Whether to use sparse layers in the wipe tower.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wipe_tower_no_sparse_layers: Option<String>,
    /// Whether to enable support structures.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_support: Option<String>,
    /// The filament to use for support structures.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_filament: Option<String>,
    /// The filament to use for support interface.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_interface_filament: Option<String>,
    /// Whether to use a loop pattern for support interface.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_interface_loop_pattern: Option<String>,
    /// The speed for printing support interface.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_interface_speed: Option<String>,
    /// The number of top layers for support interface.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_interface_top_layers: Option<String>,
    /// Whether to use adaptive layer height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adaptive_layer_height: Option<String>,
    /// Whether to disable support for bridge areas.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bridge_no_support: Option<String>,
    /// The width of the brim in mm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brim_width: Option<String>,
    /// Whether to detect thin walls.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detect_thin_wall: Option<String>,
    /// Whether to combine infill every n layers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub infill_combination: Option<String>,
    /// The direction of infill lines in degrees.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub infill_direction: Option<String>,
    /// The percentage of overlap between infill and walls.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub infill_wall_overlap: Option<String>,
    /// Whether to print additional shells on interface layers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interface_shells: Option<String>,
    /// The sequence in which parts are printed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub print_sequence: Option<String>,
    /// The identifier for these print settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub print_settings_id: Option<String>,
    /// The number of raft layers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raft_layers: Option<String>,
    /// Whether to reduce wall crossings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reduce_crossing_wall: Option<String>,
    /// The distance of the skirt from the print in mm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skirt_distance: Option<String>,
    /// Whether to use spiral mode (vase mode).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spiral_mode: Option<String>,
    /// The temperature difference for the non-active extruder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standby_temperature_delta: Option<String>,
    /// The pattern to use for support base.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_base_pattern: Option<String>,
    /// The pattern to use for support interface.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_interface_pattern: Option<String>,
    /// Whether to generate support only on the build plate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_on_build_plate_only: Option<String>,
    /// XY contour compensation value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub xy_contour_compensation: Option<String>,
    /// XY hole compensation value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub xy_hole_compensation: Option<String>,
    #[serde(flatten)]
    other: BTreeMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test deserializing nozzle diameter groups.
    #[test]
    fn test_deserialize_nozzle_diameter_group() {
        let single = r#""0.4;0.6""#;
        let range = r#"["0.4", "0.6"]"#;
        let single_group: NozzleDiameterGroup = serde_json::from_str(single).unwrap();
        let range_group: NozzleDiameterGroup = serde_json::from_str(range).unwrap();
        assert_eq!(single_group, NozzleDiameterGroup::Single("0.4;0.6".to_string()));
        assert_eq!(
            single_group.as_vec().unwrap(),
            vec![NozzleDiameter::Diameter04, NozzleDiameter::Diameter06]
        );
        assert_eq!(
            range_group,
            NozzleDiameterGroup::Range(vec![NozzleDiameter::Diameter04, NozzleDiameter::Diameter06])
        );
    }

    // Ensure we can deserialize all the filament settings.
    #[test]
    fn test_deserialize_all_filament_settings() {
        // Deserialize each file.
        for file in walkdir::WalkDir::new("profiles/BBL/filament").into_iter() {
            let file = match file {
                Ok(file) => file,
                Err(err) => panic!("Error reading file: {:?}", err),
            };
            let path = file.path();
            if path.is_dir() {
                continue;
            }
            println!("Deserializing file: {}", path.display());
            let contents = match std::fs::read_to_string(path) {
                Ok(contents) => contents,
                Err(err) => panic!("Error reading file `{}`: {:?}", path.display(), err),
            };
            if let Err(err) = serde_json::from_str::<Filament>(&contents) {
                panic!("Error deserializing file `{}` to Filament: {:?}", path.display(), err);
            }
            match serde_json::from_str::<Template>(&contents) {
                Ok(t) => {
                    if !t.other().is_empty() {
                        panic!("other settings found in file `{}`: {:?}", path.display(), t.other());
                    }
                    let new = t.load_inherited().unwrap();
                    if t.inherits().is_some() {
                        //assert!(new != *t, "Inherited settings are the same as the original: {:#?}", t);
                    } else {
                        assert_eq!(new, t, "Inherited settings are different from the original: {:#?}", t);
                    }
                }
                Err(err) => panic!("Error deserializing file `{}` to Template: {:#?}", path.display(), err),
            }
        }
    }

    // Ensure we can deserialize all the process settings.
    #[test]
    fn test_deserialize_all_process_settings() {
        // Deserialize each file.
        for file in walkdir::WalkDir::new("profiles/BBL/process").into_iter() {
            let file = match file {
                Ok(file) => file,
                Err(err) => panic!("Error reading file: {:?}", err),
            };
            let path = file.path();
            if path.is_dir() {
                continue;
            }
            println!("Deserializing file: {}", path.display());
            let contents = match std::fs::read_to_string(path) {
                Ok(contents) => contents,
                Err(err) => panic!("Error reading file `{}`: {:?}", path.display(), err),
            };
            if let Err(err) = serde_json::from_str::<Process>(&contents) {
                panic!("Error deserializing file `{}` to Process: {:?}", path.display(), err);
            }
            match serde_json::from_str::<Template>(&contents) {
                Ok(t) => {
                    if !t.other().is_empty() {
                        panic!("other settings found in file `{}`: {:?}", path.display(), t.other());
                    }
                    let new = t.load_inherited().unwrap();
                    if t.inherits().is_some() {
                        //assert!(new != *t, "Inherited settings are the same as the original: {:#?}", t);
                    } else {
                        assert_eq!(new, t, "Inherited settings are different from the original: {:#?}", t);
                    }
                }
                Err(err) => panic!("Error deserializing file `{}` to Template: {:#?}", path.display(), err),
            }
        }
    }

    // Ensure we can deserialize all the machine settings.
    #[test]
    fn test_deserialize_all_machine_settings() {
        // Deserialize each file.
        for file in walkdir::WalkDir::new("profiles/BBL/machine").into_iter() {
            let file = match file {
                Ok(file) => file,
                Err(err) => panic!("Error reading file: {:?}", err),
            };
            let path = file.path();
            if path.is_dir() {
                continue;
            }
            println!("Deserializing file: {}", path.display());
            let contents = match std::fs::read_to_string(path) {
                Ok(contents) => contents,
                Err(err) => panic!("Error reading file `{}`: {:?}", path.display(), err),
            };
            match serde_json::from_str::<Template>(&contents) {
                Ok(t) => {
                    if !t.other().is_empty() {
                        panic!("other settings found in file `{}`: {:?}", path.display(), t.other());
                    }
                    let new = t.load_inherited().unwrap();

                    if t.inherits().is_some() {
                        assert!(new != t, "Inherited settings are the same as the original: {:#?}", t);
                    } else {
                        assert_eq!(new, t, "Inherited settings are different from the original: {:#?}", t);
                    }
                }
                Err(err) => panic!("Error deserializing file `{}` to Template: {:#?}", path.display(), err),
            }
        }
    }
}
