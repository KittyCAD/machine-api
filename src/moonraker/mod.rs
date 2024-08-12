//! This module contains support for printing to moonraker 3D printers.

mod control;

use crate::Volume;
use anyhow::Result;
use moonraker::Client as MoonrakerClient;

/// Client is a connection to a Moonraker instance.
struct Client {
    client: MoonrakerClient,
    volume: Volume,
}

impl Client {
    /// Create a new Moonraker based machine. The `base_url` will be
    /// passed through to [moonraker::Client].
    pub fn new(base_url: &str, volume: Volume) -> Result<Self> {
        Ok(Self {
            volume,
            client: MoonrakerClient::new(base_url)?,
        })
    }

    /// Create a handle to a Elegoo Neptune 4.
    pub fn neptune4(base_url: &str) -> Result<Self> {
        Self::new(
            base_url,
            Volume {
                width: 255.0,
                height: 255.0,
                depth: 255.0,
            },
        )
    }
}
