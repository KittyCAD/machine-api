use anyhow::Result;
use machine_api::{
    gcode, moonraker, noop,
    slicer::{self, orca, prusa, AnySlicer},
    AnyMachineInfo, Discover, Machine, MachineInfo, MachineMakeModel, MachineType, StaticDiscover, Volume,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, future::Future, path::PathBuf, sync::Arc};
use tokio::sync::{mpsc, RwLock};

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
    fn manufacturer_model(&self) -> (Option<String>, Option<String>) {
        match self {
            Self::PrusaMk3 => (Some("Prusa".to_owned()), Some("MK3".to_owned())),
            Self::Generic => (None, None),
        }
    }

    /// return the vid/pid of the variant
    fn vid_pid(&self) -> Option<(u16, u16)> {
        match self {
            Self::PrusaMk3 => Some((0x2c99, 0x0002)),
            Self::Generic => None,
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

        /// Serial Number to match on
        serial: String,

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

impl MachineConfig {}

/// Machines that we've come across.
pub type DiscoveredMachines = Arc<RwLock<HashMap<String, RwLock<Machine>>>>;

impl Config {
    pub async fn load_model_noop(&self, machines: DiscoveredMachines) -> Result<()> {
        // Here we have no discovery required. We're just going through
        // and directly adding each noop.

        for (machine_id, machine) in self.machines.iter() {
            let MachineConfig::Noop {} = machine else {
                continue;
            };

            machines.write().await.insert(
                machine_id.clone(),
                RwLock::new(Machine::new(
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
            );
        }
        Ok(())
    }

    pub async fn load_model_usb(&self, machines: DiscoveredMachines) -> Result<()> {
        // Here we have some discovery to do. Let's collect all the configs.

        let mut known_hardware = HashMap::new();
        let mut known_configs = HashMap::new();

        for (machine_id, machine) in self.machines.iter() {
            let MachineConfig::Usb {
                baud,
                serial,
                variant,
                slicer,
            } = machine
            else {
                continue;
            };

            let (vid, pid) = variant.vid_pid().unwrap_or((0, 0));
            let (make, model) = variant.manufacturer_model();

            known_hardware.insert(
                (vid, pid, serial.clone()),
                (
                    machine_id.clone(),
                    variant.machine_type(),
                    variant.max_part_volume(),
                    *baud,
                    make,
                    model,
                ),
            );

            known_configs.insert(machine_id.clone(), machine.clone());
        }

        let usb_discover = Arc::new(gcode::UsbDiscover::new(known_hardware));

        let (found_send, found_recv) = mpsc::channel(1);
        let usb_discover1 = usb_discover.clone();
        tokio::spawn(async move {
            let usb_discover = usb_discover1;
            let _ = usb_discover.discover(found_send).await;
        });

        let machines = machines.clone();
        let usb_discover1 = usb_discover.clone();
        tokio::spawn(async move {});

        Ok(())
    }

    pub async fn load_model_moonraker(&self, machines: DiscoveredMachines) -> Result<()> {
        Ok(())
    }

    pub async fn load_models(&self, machines: DiscoveredMachines) -> Result<()> {
        self.load_model_noop(machines.clone()).await?;
        self.load_model_usb(machines.clone()).await?;
        self.load_model_moonraker(machines.clone()).await?;
        Ok(())
    }

    pub async fn load(&self) -> Result<DiscoveredMachines> {
        let machines = Arc::new(RwLock::new(HashMap::new()));
        self.load_models(machines.clone()).await?;
        Ok(machines)
    }
}
