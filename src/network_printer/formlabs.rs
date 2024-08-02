//! Formlabs backend for the [`crate::network_printer::NetworkPrinter`] trait.

use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use futures_util::{pin_mut, stream::StreamExt};

use crate::{
    config::FormLabsConfig,
    network_printer::{
        Message, NetworkPrinter, NetworkPrinterHandle, NetworkPrinterInfo, NetworkPrinterManufacturer, NetworkPrinters,
    },
};

/// The hostname formlabs printers.
const SERVICE_NAME: &str = "_formlabs_formule._tcp.local";

/// Formlabs printer backend.
pub struct Formlabs {
    pub printers: DashMap<String, NetworkPrinterHandle>,
}

impl Formlabs {
    /// Create a new Formlabs printer backend.
    pub fn new(_config: &FormLabsConfig) -> Self {
        Self {
            printers: DashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl NetworkPrinters for Formlabs {
    async fn discover(&self) -> Result<()> {
        // Iterate through responses from each Cast device, asking for new devices every 15s
        let stream = mdns::discover::all(SERVICE_NAME, std::time::Duration::from_secs(15))?.listen();
        pin_mut!(stream);

        while let Some(Ok(response)) = stream.next().await {
            if let Some(addr) = response.ip_addr() {
                let info = NetworkPrinterInfo {
                    hostname: response.hostname().map(|name| name.to_string()),
                    ip: addr,
                    port: response.port(),
                    manufacturer: NetworkPrinterManufacturer::Formlabs,
                    model: None,
                    serial: None,
                };
                let handle = NetworkPrinterHandle {
                    info,
                    client: Arc::new(Box::new(FormlabsPrinter {})),
                };
                self.printers.insert(addr.to_string(), handle);
            } else {
                println!("formlabs printer does not advertise address: {:#?}", response);
            }
        }

        anyhow::bail!("formlabs printer discovery ended unexpectedly");
    }

    fn list(&self) -> Result<Vec<NetworkPrinterInfo>> {
        Ok(self
            .printers
            .iter()
            .map(|printer| printer.value().info.clone())
            .collect())
    }

    fn list_handles(&self) -> Result<Vec<NetworkPrinterHandle>> {
        Ok(self.printers.iter().map(|printer| printer.value().clone()).collect())
    }
}

pub struct FormlabsPrinter {}

#[async_trait::async_trait]
impl NetworkPrinter for FormlabsPrinter {
    /// Get the status of a printer.
    async fn status(&self) -> Result<Message> {
        unimplemented!()
    }

    /// Pause the current print.
    async fn pause(&self) -> Result<Message> {
        unimplemented!()
    }

    /// Resume the current print.
    async fn resume(&self) -> Result<Message> {
        unimplemented!()
    }

    /// Stop the current print.
    async fn stop(&self) -> Result<Message> {
        unimplemented!()
    }

    /// Set the led on or off.
    async fn set_led(&self, _on: bool) -> Result<Message> {
        unimplemented!()
    }

    /// Get the accessories.
    async fn accessories(&self) -> Result<Message> {
        unimplemented!()
    }

    /// Slice a file.
    /// Returns the path to the sliced file.
    async fn slice(&self, _file: &std::path::Path) -> Result<std::path::PathBuf> {
        unimplemented!()
    }

    /// Print a file.
    async fn print(&self, _job_name: &str, _file: &std::path::Path) -> Result<Message> {
        unimplemented!()
    }
}
