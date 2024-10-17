//! This module contains support for printing to moonraker 3D printers.

mod control;
mod temperature;
mod variants;

use anyhow::Result;
pub use control::MachineInfo;
use moonraker::Client as MoonrakerClient;
use serde::{Deserialize, Serialize};
pub use variants::MoonrakerVariant;

use crate::{slicer, MachineMakeModel, Volume};

/// Configuration information for a Moonraker-based endpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Slicer to use with this printer
    pub slicer: slicer::Config,

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
}

impl Client {
    /// Create a new Moonraker based machine. The `base_url` will be
    /// passed through to [moonraker::Client].
    pub fn new(base_url: &str, make_model: MachineMakeModel, volume: Option<Volume>) -> Result<Self> {
        Ok(Self {
            make_model,
            volume,
            client: MoonrakerClient::new(base_url)?,
        })
    }

    /// Return the underling [MoonrakerClient].
    pub fn get_client(&self) -> &MoonrakerClient {
        &self.client
    }
}
