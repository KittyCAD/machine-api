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

impl MachineHandle {
    pub async fn slice_and_print(&self, file: &std::path::Path) -> anyhow::Result<()> {
        match self {
            MachineHandle::UsbPrinter(printer) => {
                let mut machine = crate::usb_printer::UsbPrinter::new(printer.clone());
                machine.slice_and_print(file)?;
            }
            MachineHandle::NetworkPrinter(printer) => {
                let result = printer.client.slice_and_print(file).await?;
                println!("{:?}", result);
            }
        }
        Ok(())
    }
}

impl From<MachineHandle> for Machine {
    fn from(handle: MachineHandle) -> Self {
        match handle {
            MachineHandle::UsbPrinter(printer) => Machine::UsbPrinter(printer),
            MachineHandle::NetworkPrinter(printer) => Machine::NetworkPrinter(printer.info),
        }
    }
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
