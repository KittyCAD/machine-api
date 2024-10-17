//! Templates for the machine, filament, and process settings.

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::message::NozzleDiameter;

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
    pub from: String,
    /// The unique identifier for the machine's settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setting_id: Option<String>,
    /// The instantiation details of the machine.
    pub instantiation: String,
    /// A list of nozzle diameters supported by the machine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nozzle_diameter: Option<NozzleDiameterGroup>,
    /// The current bed type of the machine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub curr_bed_type: Option<String>,
    /// The model of the printer.
    pub printer_model: String,
    /// The variant of the printer.
    pub printer_variant: String,
    /// Areas of the bed to exclude from printing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bed_exclude_area: Vec<String>,
    /// Default filament profiles for the machine.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_filament_profile: Vec<String>,
    /// Default print profile for the machine.
    pub default_print_profile: String,
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
}

/// The filament settings template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Filament {
    /// The name of the filament.
    pub name: String,
    /// The inheritance information of the filament.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherits: Option<String>,
    /// The origin or source of the filament.
    pub from: String,
    /// The unique identifier for the filament's settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setting_id: Option<String>,
    /// The instantiation details of the filament.
    pub instantiation: String,
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
    pub from: String,
    /// The unique identifier for the process settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setting_id: Option<String>,
    /// The instantiation details of the process.
    pub instantiation: String,
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
        for file in walkdir::WalkDir::new("../profiles/BBL/filament").into_iter() {
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
            if let Err(err) = serde_json::from_str::<Template>(&contents) {
                panic!("Error deserializing file `{}` to Template: {:?}", path.display(), err);
            }
        }
    }

    // Ensure we can deserialize all the process settings.
    #[test]
    fn test_deserialize_all_process_settings() {
        // Deserialize each file.
        for file in walkdir::WalkDir::new("../profiles/BBL/process").into_iter() {
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
            if let Err(err) = serde_json::from_str::<Template>(&contents) {
                panic!("Error deserializing file `{}` to Template: {:?}", path.display(), err);
            }
        }
    }

    // Ensure we can deserialize all the machine settings.
    #[test]
    fn test_deserialize_all_machine_settings() {
        // Deserialize each file.
        for file in walkdir::WalkDir::new("../profiles/BBL/machine").into_iter() {
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
            if let Err(err) = serde_json::from_str::<Machine>(&contents) {
                panic!("Error deserializing file `{}` to Machine: {:?}", path.display(), err);
            }
            if let Err(err) = serde_json::from_str::<Template>(&contents) {
                panic!("Error deserializing file `{}` to Template: {:?}", path.display(), err);
            }
        }
    }
}
