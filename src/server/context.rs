use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use tokio::sync::Mutex;

use crate::{
    config::Config,
    network_printer::{NetworkPrinterManufacturer, NetworkPrinters},
};

/**
 * Application-specific context (state shared by handler functions)
 */
pub struct Context {
    pub schema: serde_json::Value,
    pub logger: slog::Logger,
    pub usb_printers: Arc<HashMap<String, crate::usb_printer::UsbPrinterInfo>>,
    pub network_printers: Arc<HashMap<NetworkPrinterManufacturer, Box<dyn NetworkPrinters>>>,
    pub active_jobs: Mutex<HashMap<String, tokio::task::JoinHandle<Result<crate::machine::Message>>>>,
}

impl Context {
    /**
     * Return a new Context.
     */
    pub async fn new(config: &Config, schema: serde_json::Value, logger: slog::Logger) -> Result<Context> {
        let mut network_printers: HashMap<NetworkPrinterManufacturer, Box<dyn NetworkPrinters>> = HashMap::new();

        if let Some(formlabs_config) = &config.formlabs {
            // Add formlabs backend.
            network_printers.insert(
                NetworkPrinterManufacturer::Formlabs,
                Box::new(crate::network_printer::formlabs::Formlabs::new(formlabs_config)),
            );
        }

        if let Some(bambulabs_config) = &config.bambulabs {
            // Add Bambu Lab backend.
            network_printers.insert(
                NetworkPrinterManufacturer::Bambu,
                Box::new(crate::network_printer::bambu_x1_carbon::BambuX1Carbon::new(
                    bambulabs_config,
                )),
            );
        }

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

    pub fn list_machine_handles(&self) -> Result<HashMap<String, crate::machine::MachineHandle>> {
        let mut machines: HashMap<String, crate::machine::MachineHandle> = HashMap::new();
        for (_, np) in self.network_printers.iter() {
            for np in np.list_handles()? {
                if let Some(hostname) = np.info.hostname.clone() {
                    machines.insert(hostname, np.into());
                } else {
                    machines.insert(np.info.ip.to_string(), np.into());
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

    pub fn find_machine_handle_by_id(&self, id: &str) -> Result<Option<crate::machine::MachineHandle>> {
        self.list_machine_handles().map(|machines| machines.get(id).cloned())
    }
}

pub async fn discovery(ctx: Arc<Context>, dur: tokio::time::Duration) -> Result<()> {
    println!("Discovering printers...");
    // We don't care if it times out, we just want to wait for the discovery tasks to
    // finish.
    let _ = tokio::time::timeout(dur, async move {
        let form_labs = ctx
            .network_printers
            .get(&NetworkPrinterManufacturer::Formlabs)
            .expect("No formlabs discover task registered");
        let bambu = ctx
            .network_printers
            .get(&NetworkPrinterManufacturer::Bambu)
            .expect("No Bambu discover task registered");

        tokio::join!(form_labs.discover(), bambu.discover())
    })
    .await;

    Ok(())
}
