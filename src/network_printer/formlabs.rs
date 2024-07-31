//! Formlabs backend for the [`crate::network_printer::NetworkPrinter`] trait.

use anyhow::Result;
use dashmap::DashMap;
use futures_util::{pin_mut, stream::StreamExt};

use crate::network_printer::{NetworkPrinter, NetworkPrinterInfo, NetworkPrinterManufacturer};

/// The hostname formlabs printers.
const SERVICE_NAME: &str = "_formlabs_formule._tcp.local";

/// Formlabs printer backend.
pub struct Formlabs {
    pub printers: DashMap<String, NetworkPrinterInfo>,
}

impl Formlabs {
    /// Create a new Formlabs printer backend.
    pub fn new() -> Self {
        Self {
            printers: DashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl NetworkPrinter for Formlabs {
    async fn discover(&self) -> Result<()> {
        // Iterate through responses from each Cast device, asking for new devices every 15s
        let stream = mdns::discover::all(SERVICE_NAME, std::time::Duration::from_secs(15))?.listen();
        pin_mut!(stream);

        while let Some(Ok(response)) = stream.next().await {
            if let Some(addr) = response.ip_addr() {
                let printer = NetworkPrinterInfo {
                    hostname: response.hostname().map(|name| name.to_string()),
                    ip: addr,
                    port: response.port(),
                    manufacturer: NetworkPrinterManufacturer::Formlabs,
                    model: None,
                    serial: None,
                };
                self.printers.insert(addr.to_string(), printer);
            } else {
                println!("formlabs printer does not advertise address: {:#?}", response);
            }
        }

        anyhow::bail!("formlabs printer discovery ended unexpectedly");
    }

    fn list(&self) -> Result<Vec<NetworkPrinterInfo>> {
        Ok(self.printers.iter().map(|printer| printer.value().clone()).collect())
    }
}
