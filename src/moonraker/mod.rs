//! This module contains support for printing to moonraker 3D printers.

mod control;
mod slicer;

pub use control::MachineInfo;

use crate::{
    slicer::{prusa, AnySlicer},
    Volume,
};
use anyhow::Result;
use moonraker::Client as MoonrakerClient;
use std::path::PathBuf;

/// Client is a connection to a Moonraker instance.
pub struct Client {
    client: MoonrakerClient,
    volume: Volume,
    slicer: AnySlicer,
}

impl Client {
    /// Create a new Moonraker based machine. The `base_url` will be
    /// passed through to [moonraker::Client].
    pub fn new(slicer: AnySlicer, base_url: &str, volume: Volume) -> Result<Self> {
        Ok(Self {
            slicer,
            volume,
            client: MoonrakerClient::new(base_url)?,
        })
    }

    /// Create a handle to a Elegoo Neptune 4.
    pub fn neptune4(slicer_cfg: &str, base_url: &str) -> Result<Self> {
        let slicer_cfg: PathBuf = slicer_cfg.parse()?;

        Self::new(
            prusa::Slicer::new(&slicer_cfg).into(),
            base_url,
            Volume {
                width: 255.0,
                height: 255.0,
                depth: 255.0,
            },
        )
    }
}
