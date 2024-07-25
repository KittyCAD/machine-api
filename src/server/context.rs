use std::collections::HashMap;

use anyhow::Result;
use tokio::sync::Mutex;

use crate::usb_printer::UsbPrinterList;

/**
 * Application-specific context (state shared by handler functions)
 */
pub struct Context {
    pub schema: serde_json::Value,
    pub logger: slog::Logger,
    pub settings: crate::Server,
    pub usb_printers: UsbPrinterList,
    pub active_jobs: Mutex<HashMap<String, tokio::task::JoinHandle<anyhow::Result<()>>>>,
}

impl Context {
    /**
     * Return a new Context.
     */
    pub async fn new(schema: serde_json::Value, logger: slog::Logger, settings: crate::Server) -> Result<Context> {
        // Create the context.
        Ok(Context {
            schema,
            logger,
            settings,
            usb_printers: crate::usb_printer::UsbPrinter::list_all(),
            active_jobs: Mutex::new(HashMap::new()),
        })
    }
}
