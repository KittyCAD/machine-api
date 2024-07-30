use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use tokio::sync::Mutex;

use crate::{
    network_printer::{NetworkPrinter, NetworkPrinterManufacturer},
    usb_printer::UsbPrinterList,
};

/**
 * Application-specific context (state shared by handler functions)
 */
pub struct Context {
    pub schema: serde_json::Value,
    pub logger: slog::Logger,
    pub settings: crate::Server,
    pub usb_printers: UsbPrinterList,
    pub network_printers: Arc<HashMap<NetworkPrinterManufacturer, Box<dyn NetworkPrinter>>>,
    pub active_jobs: Mutex<HashMap<String, tokio::task::JoinHandle<anyhow::Result<()>>>>,
}

impl Context {
    /**
     * Return a new Context.
     */
    pub async fn new(schema: serde_json::Value, logger: slog::Logger, settings: crate::Server) -> Result<Context> {
        let mut network_printers: HashMap<NetworkPrinterManufacturer, Box<dyn NetworkPrinter>> = HashMap::new();

        // Add formlabs backend.
        network_printers.insert(
            NetworkPrinterManufacturer::Formlabs,
            Box::new(crate::network_printer::formlabs::Formlabs::new()),
        );

        // Create the context.
        Ok(Context {
            schema,
            logger,
            settings,
            network_printers: Arc::new(network_printers),
            usb_printers: crate::usb_printer::UsbPrinter::list_all(),
            active_jobs: Mutex::new(HashMap::new()),
        })
    }
}
