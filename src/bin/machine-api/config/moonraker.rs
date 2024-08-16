use super::{Config, MachineConfig, SlicerConfig};
use anyhow::Result;
use machine_api::{
    gcode::{UsbDiscover, UsbHardwareMetadata},
    moonraker::Client,
    AnyMachine, Machine, MachineMakeModel, MachineType, StaticDiscover, Volume,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Specific make/model of device we're connected to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MoonrakerVariant {
    /// Elegoo Neptune 4
    ElegooNeptune4,

    /// Generic moonraker printer.
    Generic,
}

impl MoonrakerVariant {
    /// Return the [MachineType] of the variant.
    pub fn machine_type(&self) -> MachineType {
        match self {
            MoonrakerVariant::ElegooNeptune4 => MachineType::FusedDeposition,
            MoonrakerVariant::Generic => MachineType::FusedDeposition,
        }
    }

    /// Return the max part volume.
    pub fn max_part_volume(&self) -> Option<Volume> {
        match self {
            MoonrakerVariant::ElegooNeptune4 => None,
            MoonrakerVariant::Generic => None,
        }
    }

    /// Return the make and model.
    pub fn manufacturer_model(&self) -> (Option<String>, Option<String>) {
        match self {
            MoonrakerVariant::ElegooNeptune4 => (Some("Elegoo".to_string()), Some("Neptune 4".to_string())),
            MoonrakerVariant::Generic => (None, None),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MachineConfigMoonraker {
    slicer: SlicerConfig,
    variant: MoonrakerVariant,
    endpoint: String,
}

impl MachineConfigMoonraker {
    pub fn get_key(&self) -> String {
        self.endpoint.clone()
    }
}

impl Config {
    /// Load a StaticDiscover stub based on the provided machine config.
    pub async fn load_discover_moonraker(&self) -> Result<Option<StaticDiscover>> {
        let mut all_machines: Vec<Arc<Mutex<AnyMachine>>> = vec![];
        for (_machine_key, machine) in self
            .machines
            .iter()
            .map(|(machine_key, machine)| {
                if let MachineConfig::Moonraker(moonraker) = machine {
                    Some((machine_key, moonraker))
                } else {
                    None
                }
            })
            .flatten()
        {
            let slicer = machine.slicer.load().await?;
            let (manufacturer, model) = machine.variant.manufacturer_model();
            let volume = machine.variant.max_part_volume();

            let moonraker = Client::new(
                &machine.endpoint,
                MachineMakeModel {
                    manufacturer,
                    model,
                    serial: None,
                },
                volume,
            )?;
            all_machines.push(Arc::new(Mutex::new(moonraker.into())));
        }

        if !all_machines.is_empty() {
            Ok(Some(StaticDiscover::new(all_machines)))
        } else {
            Ok(None)
        }
    }
}
