//! This module contains support for printing to Bambu Lab 3D printers.

mod control;

pub use control::X1Carbon;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

    /// Get the machine config for the given id.
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
