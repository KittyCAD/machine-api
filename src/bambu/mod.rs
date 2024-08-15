//! This module contains support for printing to Bambu Lab 3D printers.

mod control;
mod discover;

pub use discover::Discover;

use crate::MachineMakeModel;
use bambulabs::client::Client;
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, path::PathBuf, sync::Arc};

/// Control channel handle to a Bambu Labs X1 Carbon.
#[derive(Clone)]
pub struct X1Carbon {
    client: Arc<Client>,
    info: PrinterInfo,
}

/// Information regarding a discovered X1 Carbon.
#[derive(Debug, Clone)]
pub struct PrinterInfo {
    /// Make and model of the PrinterInfo. This is accessed through the
    /// `MachineMakeModel` trait.
    make_model: MachineMakeModel,

    /// The hostname of the printer.
    pub hostname: Option<String>,

    /// The IP address of the printer.
    pub ip: IpAddr,

    /// The port of the printer.
    pub port: Option<u16>,
}

/// The configuration for bambu labs printers.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// List of all known printers for which we have access code and
    /// configurations defined.
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
    /// The printer's id.
    pub id: String,

    /// The access code for the printer.
    pub access_code: String,

    /// The slicer configuration for the printer.
    pub slicer_config: PathBuf,
}
