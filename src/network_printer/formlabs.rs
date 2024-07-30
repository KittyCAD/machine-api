//! Formlabs backend for the [`crate::network_printer::NetworkPrinter`] trait.

use anyhow::Result;
use dashmap::DashMap;
use futures_util::{pin_mut, stream::StreamExt};

use crate::network_printer::{to_ip_addr, NetworkPrinter, NetworkPrinterInfo, NetworkPrinterManufacturer};

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
            let addr = response.records().filter_map(to_ip_addr).next();

            if let Some(addr) = addr {
                println!("found formlabs printer at {}", addr);
                self.printers.insert(
                    addr.to_string(),
                    NetworkPrinterInfo {
                        ip: addr,
                        manufacturer: NetworkPrinterManufacturer::Formlabs,
                        model: "Formule".to_string(),
                    },
                );
            } else {
                println!("formlabs printer does not advertise address");
            }
        }

        anyhow::bail!("formlabs printer discovery ended unexpectedly");
    }
}
