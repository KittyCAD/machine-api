//! A trait for a printer on a network.

pub mod bambu_x1_carbon;
pub mod formlabs;

use std::{fmt::Debug, net::IpAddr, sync::Arc};

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A network printers interface.
#[async_trait::async_trait]
pub trait NetworkPrinters: Send + Sync {
    /// Discover all printers on the network.
    /// This will continuously search for printers until the program is stopped.
    /// You likely want to spawn this on a separate thread.
    async fn discover(&self) -> Result<()>;

    /// List all printers found on the network.
    fn list(&self) -> Result<Vec<NetworkPrinterInfo>>;

    /// List all printer handles found on the network.
    fn list_handles(&self) -> Result<Vec<NetworkPrinterHandle>>;
}

/// A network printers interface.
#[async_trait::async_trait]
pub trait NetworkPrinter: Send + Sync {
    /// Get the status of a printer.
    async fn status(&self) -> Result<Message>;

    /// Get the version of the printer.
    async fn version(&self) -> Result<Message>;

    /// Pause the current print.
    async fn pause(&self) -> Result<Message>;

    /// Resume the current print.
    async fn resume(&self) -> Result<Message>;

    /// Stop the current print.
    async fn stop(&self) -> Result<Message>;

    /// Set the led on or off.
    async fn set_led(&self, on: bool) -> Result<Message>;

    /// Get the accessories.
    async fn accessories(&self) -> Result<Message>;

    /// Slice a file.
    /// Returns the path to the sliced file.
    async fn slice(&self, file: &std::path::Path) -> Result<std::path::PathBuf>;

    /// Print a file.
    async fn print(&self, job_name: &str, file: &std::path::Path) -> Result<Message>;

    /// Slice and print a file.
    async fn slice_and_print(&self, job_name: &str, file: &std::path::Path) -> Result<Message> {
        let sliced = self.slice(file).await?;
        self.print(job_name, &sliced).await
    }
}

/// Handle for a 3d printer.
#[derive(Clone)]
pub struct NetworkPrinterHandle {
    /// Information for the printer.
    pub info: NetworkPrinterInfo,
    /// The client interface for the printer.
    pub client: Arc<Box<dyn NetworkPrinter>>,
}

/// Details for a 3d printer connected over network.
#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
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
    /// The serial number of the printer.
    pub serial: Option<String>,
}

/// Network printer manufacturer.
#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum NetworkPrinterManufacturer {
    /// Bambu.
    Bambu,
    /// Formlabs.
    Formlabs,
}

/// A message from the printer.
#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
pub enum Message {
    Bambu(bambulabs::message::Message),
    Formlabs {},
}

impl From<bambulabs::message::Message> for Message {
    fn from(msg: bambulabs::message::Message) -> Self {
        Self::Bambu(msg)
    }
}
