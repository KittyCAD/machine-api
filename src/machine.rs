use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{network_printer::NetworkPrinterInfo, usb_printer::UsbPrinterInfo};

/// Details for a 3d printer connected over USB.
#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
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

/// A handle for machines with their client traits.
#[derive(Clone)]
pub enum MachineHandle {
    UsbPrinter(crate::usb_printer::UsbPrinterInfo),
    NetworkPrinter(crate::network_printer::NetworkPrinterHandle),
}

impl From<UsbPrinterInfo> for MachineHandle {
    fn from(printer: UsbPrinterInfo) -> Self {
        MachineHandle::UsbPrinter(printer)
    }
}

impl From<crate::network_printer::NetworkPrinterHandle> for MachineHandle {
    fn from(printer: crate::network_printer::NetworkPrinterHandle) -> Self {
        MachineHandle::NetworkPrinter(printer)
    }
}
