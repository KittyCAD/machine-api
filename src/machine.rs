use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{network_printer::NetworkPrinterInfo, usb_printer::UsbPrinterInfo};

/// Details for a 3d printer connected over USB.
#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Machine {
    UsbPrinter(UsbPrinterInfo),
    NetworkPrinter(NetworkPrinterInfo),
}

impl From<UsbPrinterInfo> for Machine {
    fn from(printer: UsbPrinterInfo) -> Self {
        Machine::UsbPrinter(printer)
    }
}

impl From<NetworkPrinterInfo> for Machine {
    fn from(printer: NetworkPrinterInfo) -> Self {
        Machine::NetworkPrinter(printer)
    }
}

impl Machine {
    pub fn id(&self) -> String {
        match self {
            Machine::UsbPrinter(printer) => printer.id.clone(),
            Machine::NetworkPrinter(printer) => printer.hostname.clone().unwrap_or_else(|| printer.ip.to_string()),
        }
    }
}
