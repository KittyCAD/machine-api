use anyhow::Result;
use machine_api::{
    bambu, moonraker, noop,
    slicer::{self, orca, prusa, AnySlicer},
    Machine, MachineMakeModel, MachineType, Volume,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio_serial::SerialPortBuilderExt;

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

/// Specific make/model of device we're connected to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MoonrakerVariant {
    /// Elegoo Neptune 4
    Neptune4,

    /// Generic printer.
    Generic,
}

/// Specific make/model of Bambu Labs printer.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum BambuVariant {
    /// Bambu Labs X1 Carbon
    X1Carbon,
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
        /// Path on the filesystem to connect to the printer (like
        /// `/dev/ttyUSB0` or `/dev/ttyACM2`).
        port: String,

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

    /// Configure a Bambu X1 Carbon
    Bambu {
        /// Specific model of Bambu printer.
        variant: BambuVariant,

        /// Access code for this printer.
        access_code: String,

        /// Use the configured slicer.
        slicer: SlicerConfig,
    },
}

impl MachineConfig {
    async fn load(&self, _ctx: &LoadContext) -> Result<Machine> {
        match self {
            Self::Usb {
                port,
                baud,
                variant,
                slicer,
            } => {
                let slicer = slicer.load().await?;
                let port = tokio_serial::new(port, *baud).open_native_async()?;
                let usb = match variant {
                    UsbVariant::Generic => machine_api::gcode::Usb::new(
                        port,
                        MachineType::FusedDeposition,
                        None,
                        MachineMakeModel {
                            manufacturer: None,
                            model: None,
                            serial: None,
                        },
                    ),
                    UsbVariant::PrusaMk3 => machine_api::gcode::Usb::prusa_mk3(port),
                };
                Ok(Machine::new(usb, slicer))
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
            Self::Bambu {
                variant,
                access_code,
                slicer,
            } => {
                let slicer = slicer.load();
                // let discover = ctx.bambu_discover.clone();

                unimplemented!();
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

/// Used by the Load calls to pass around required state.
struct LoadContext {
    bambu_discover: Option<Arc<bambu::Discover>>,
}

impl Config {
    async fn spawn_bambu_discover(&self) -> Option<Arc<bambu::Discover>> {
        let machines: Vec<bambu::MachineConfig> = self
            .machines
            .iter()
            .map(|(machine_id, machine)| {
                if let MachineConfig::Bambu { access_code, .. } = machine {
                    return Some(bambu::MachineConfig {
                        id: machine_id.clone(),
                        access_code: access_code.clone(),
                    });
                }
                None
            })
            .flatten()
            .collect();

        if machines.is_empty() {
            return None;
        }

        let discover = Arc::new(bambu::Discover::new(&bambu::Config { machines }));
        let discover1 = discover.clone();
        tokio::spawn(async move {
            let discover = discover1;
            tracing::info!("starting bambu discovery");
            discover.clone().discover();
        });

        Some(discover)
    }

    pub async fn load(&self) -> Result<HashMap<String, Machine>> {
        let ctx = LoadContext {
            bambu_discover: self.spawn_bambu_discover().await,
        };

        let mut machines = HashMap::new();
        for (machine_id, machine_config) in self.machines.iter() {
            machines.insert(machine_id.clone(), machine_config.load(&ctx).await?);
        }
        Ok(machines)
    }
}
