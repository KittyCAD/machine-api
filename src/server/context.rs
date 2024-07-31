use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use tokio::sync::Mutex;

use crate::network_printer::{NetworkPrinter, NetworkPrinterManufacturer};

/**
 * Application-specific context (state shared by handler functions)
 */
pub struct Context {
    pub schema: serde_json::Value,
    pub logger: slog::Logger,
    pub usb_printers: Arc<HashMap<String, crate::usb_printer::UsbPrinterInfo>>,
    pub network_printers: Arc<HashMap<NetworkPrinterManufacturer, Box<dyn NetworkPrinter>>>,
    pub active_jobs: Mutex<HashMap<String, tokio::task::JoinHandle<anyhow::Result<()>>>>,
}

impl Context {
    /**
     * Return a new Context.
     */
    pub async fn new(schema: serde_json::Value, logger: slog::Logger) -> Result<Context> {
        let mut network_printers: HashMap<NetworkPrinterManufacturer, Box<dyn NetworkPrinter>> = HashMap::new();

        // Add formlabs backend.
        network_printers.insert(
            NetworkPrinterManufacturer::Formlabs,
            Box::new(crate::network_printer::formlabs::Formlabs::new()),
        );

        // Add Bambu Lab backend.
        network_printers.insert(
            NetworkPrinterManufacturer::Bambu,
            Box::new(crate::network_printer::bambu_x1_carbon::BambuX1Carbon::new()),
        );

        // Create the context.
        Ok(Context {
            schema,
            logger,
            network_printers: Arc::new(network_printers),
            usb_printers: Arc::new(crate::usb_printer::UsbPrinter::list_all()),
            active_jobs: Mutex::new(HashMap::new()),
        })
    }

    pub fn list_machines(&self) -> Result<HashMap<String, crate::machine::Machine>> {
        let mut machines: HashMap<String, crate::machine::Machine> = HashMap::new();
        for (_, np) in self.network_printers.iter() {
            for np in np.list()? {
                if let Some(hostname) = np.hostname.clone() {
                    machines.insert(hostname, np.into());
                } else {
                    machines.insert(np.ip.to_string(), np.into());
                }
            }
        }
        for (_, up) in self.usb_printers.iter() {
            machines.insert(up.id.clone(), up.clone().into());
        }
        Ok(machines)
    }

    pub fn find_machine_by_id(&self, id: &str) -> Result<Option<crate::machine::Machine>> {
        self.list_machines().map(|machines| machines.get(id).cloned())
    }
}
