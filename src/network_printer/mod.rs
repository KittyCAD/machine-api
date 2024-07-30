//! A trait for a printer on a network.

pub mod formlabs;

use std::net::IpAddr;

use anyhow::Result;
use schemars::JsonSchema;
use serde::Serialize;

/// A network printer interface.
#[async_trait::async_trait]
pub trait NetworkPrinter: Send + Sync {
    /// Discover all printers on the network.
    /// This will continuously search for printers until the program is stopped.
    /// You likely want to spawn this on a separate thread.
    async fn discover(&self) -> Result<()>;

    // Print a file.
    // fn print(&self);
}

/// Details for a 3d printer connected over USB.
#[derive(Clone, Debug, JsonSchema, Serialize)]
pub struct NetworkPrinterInfo {
    /// The hostname of the printer.
    pub hostname: Option<String>,
    /// The IP address of the printer.
    pub ip: IpAddr,
    /// The port of the printer.
    pub port: Option<u16>,
    /// The manufacturer of the printer.
    pub manufacturer: NetworkPrinterManufacturer,
    /// The model of the printer.
    pub model: Option<String>,
}

/// Network printer manufacturer.
#[derive(Clone, Debug, JsonSchema, Serialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum NetworkPrinterManufacturer {
    /// Bambu.
    Bambu,
    /// Formlabs.
    Formlabs,
}
