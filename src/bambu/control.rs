use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

/// The configuration for bambu labs machines.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// The machine ids and access codes for communication of LAN.
    pub machines: Vec<MachineConfig>,
}

impl Config {
    /// Get the access code for a machine.
    pub fn get_access_code(&self, id: &str) -> Option<String> {
        self.machines.iter().find(|m| m.id == id).map(|m| m.access_code.clone())
    }

    // Get the machine config for the given id.
    pub fn get_machine_config(&self, id: &str) -> Option<&MachineConfig> {
        self.machines.iter().find(|m| m.id == id)
    }
}

/// The configuration for a single bambu labs machine.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MachineConfig {
    /// The machine id.
    pub id: String,
    /// The access code for the machine.
    pub access_code: String,
    /// The slicer configuration for the machine.
    pub slicer_config: PathBuf,
}

pub struct X1Carbon {
    pub client: Arc<bambulabs::client::Client>,
}

impl X1Carbon {
    /// Get the latest status of the printer.
    pub fn get_status(&self) -> Result<Option<bambulabs::message::PushStatus>> {
        self.client.get_status()
    }

    /// Check if the printer has an AMS.
    pub fn has_ams(&self) -> Result<bool> {
        let Some(status) = self.get_status()? else {
            return Ok(false);
        };

        let Some(ams) = status.ams else {
            return Ok(false);
        };

        let Some(ams_exists) = ams.ams_exist_bits else {
            return Ok(false);
        };

        Ok(ams_exists != "0")
    }
}
