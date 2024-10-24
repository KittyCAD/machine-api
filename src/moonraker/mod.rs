//! This module contains support for printing to moonraker 3D printers.

mod control;
mod temperature;
mod variants;

use anyhow::Result;
pub use control::MachineInfo;
use moonraker::Client as MoonrakerClient;
use serde::{Deserialize, Serialize};
pub use temperature::TemperatureSensors;
pub use variants::MoonrakerVariant;

use crate::{slicer, Filament, MachineMakeModel, Volume};

/// Configuration information for a Moonraker-based endpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Slicer to use with this printer
    pub slicer: slicer::Config,

    /// Extrusion hotend nozzle's diameter.
    pub nozzle_diameter: f64,

    /// Available filaments.
    pub filaments: Vec<Filament>,

    /// Currently loaded filament, if possible to determine.
    pub loaded_filament_idx: Option<usize>,

    /// Specific make/model of Moonraker-based printer.
    pub variant: MoonrakerVariant,

    /// HTTP URL to use for this printer.
    pub endpoint: String,
}

/// Client is a connection to a Moonraker instance.
#[derive(Clone)]
pub struct Client {
    client: MoonrakerClient,
    make_model: MachineMakeModel,
    volume: Option<Volume>,

    config: Config,
}

impl Client {
    /// Create a new Moonraker based machine. The `base_url` will be
    /// passed through to [moonraker::Client].
    pub fn new(config: &Config, make_model: MachineMakeModel) -> Result<Self> {
        Ok(Self {
            make_model,
            volume: config.variant.get_max_part_volume(),
            client: MoonrakerClient::new(&config.endpoint)?,
            config: config.clone(),
        })
    }

    /// Return the underling [MoonrakerClient].
    pub fn get_client(&self) -> &MoonrakerClient {
        &self.client
    }

    /// Return the underling [Config]
    pub(crate) fn get_config(&self) -> &Config {
        &self.config
    }
}
