use anyhow::Result;
use machine_api::{
    gcode, moonraker, noop,
    slicer::{self, orca, prusa, AnySlicer},
    Machine, MachineMakeModel, MachineType, Volume,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    machines: HashMap<String, MachineConfig>,
}

/// Specific make/model of device we're connected to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum UsbVariant {
    /// Prusa Labs MK3.
    PrusaMk3,

    /// Generic gcode-based printer.
    Generic,
}

impl UsbVariant {
    /// return the machine type of the variant
    pub fn machine_type(&self) -> MachineType {
        MachineType::FusedDeposition
    }

    /// return the make/model of the variant
    pub fn manufacturer_model(&self) -> (Option<String>, Option<String>) {
        match self {
            Self::PrusaMk3 => (Some("Prusa".to_owned()), Some("MK3".to_owned())),
            Self::Generic => (None, None),
        }
    }

    /// return the max part volume of the variant
    pub fn max_part_volume(&self) -> Option<Volume> {
        match self {
            Self::PrusaMk3 => Some(Volume {
                width: 250.0,
                depth: 210.0,
                height: 210.0,
            }),
            Self::Generic => None,
        }
    }
}

/// Specific make/model of device we're connected to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MoonrakerVariant {
    /// Elegoo Neptune 4
    Neptune4,

    /// Generic printer.
    Generic,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SlicerConfig {
    /// Use the Prusa Slicer.
    Prusa { config: String },

    /// Use the Orca Slicer.
    Orca { config: String },
}

impl SlicerConfig {
    pub async fn load(&self) -> Result<AnySlicer> {
        Ok(match self {
            Self::Prusa { config } => {
                let path: PathBuf = config.parse().unwrap();
                prusa::Slicer::new(&path).into()
            }
            Self::Orca { config } => {
                let path: PathBuf = config.parse().unwrap();
                orca::Slicer::new(&path).into()
            }
        })
    }
}

/// Specific machine configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum MachineConfig {
    /// Direct USB connection to the printer.
    Usb {
        /// Baud rate that the printer operates at.
        baud: u32,

        /// Specific USB device (or generic!)
        variant: UsbVariant,

        /// Use the configured slicer.
        slicer: SlicerConfig,
    },

    /// Connect to a Moonraker-based Klipper 3D printer over the network.
    Moonraker {
        /// HTTP endpoint to talk with
        endpoint: String,

        /// Specific Moonraker-based device (or generic!)
        variant: MoonrakerVariant,

        /// Use the configured slicer.
        slicer: SlicerConfig,
    },

    /// Use a no-op printer backend.
    Noop {},
}

impl MachineConfig {
    pub async fn load(&self) -> Result<Machine> {
        match self {
            Self::Usb { baud, variant, slicer } => {
                let (manufacturer, model) = variant.manufacturer_model();

                gcode::UsbDiscover::new(HashMap::from([
                    // foo
                    (
                        (0x1a86u16, 0x7523u16, "0".to_string()),
                        (
                            variant.machine_type(),
                            variant.max_part_volume().clone(),
                            *baud,
                            manufacturer,
                            model,
                        ),
                    ),
                ]));

                let slicer = slicer.load().await?;

                unimplemented!();
            }
            Self::Moonraker {
                endpoint,
                slicer,
                variant,
            } => {
                let slicer = slicer.load().await?;
                let machine = match variant {
                    MoonrakerVariant::Generic => moonraker::Client::new(
                        endpoint,
                        MachineMakeModel {
                            manufacturer: None,
                            model: None,
                            serial: None,
                        },
                        None,
                    )?,
                    MoonrakerVariant::Neptune4 => moonraker::Client::neptune4(endpoint)?,
                };
                Ok(Machine::new(machine, slicer))
            }
            Self::Noop {} => Ok(Machine::new(
                noop::Noop::new(
                    MachineMakeModel {
                        manufacturer: Some("Zoo Corporation".to_string()),
                        model: Some("No-op Machine!".to_string()),
                        serial: Some("cheerios".to_string()),
                    },
                    MachineType::FusedDeposition,
                    Some(Volume {
                        width: 500.0,
                        height: 600.0,
                        depth: 700.0,
                    }),
                ),
                slicer::noop::Slicer::new(),
            )),
        }
    }
}

impl Config {
    pub async fn load(&self) -> Result<HashMap<String, Machine>> {
        let mut machines = HashMap::new();
        for (machine_id, machine_config) in self.machines.iter() {
            machines.insert(machine_id.clone(), machine_config.load().await?);
        }
        Ok(machines)
    }
}
