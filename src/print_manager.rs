use std::path::PathBuf;

use crate::machine::MachineHandle;

pub struct PrintJob {
    file: PathBuf,
    machine: MachineHandle,
}

impl PrintJob {
    pub fn new(file: PathBuf, machine: MachineHandle) -> Self {
        Self { file, machine }
    }

    pub async fn spawn(&self) -> tokio::task::JoinHandle<anyhow::Result<()>> {
        let file = self.file.clone();
        let machine = self.machine.clone();
        tokio::task::spawn(async move {
            match machine {
                MachineHandle::UsbPrinter(_printer) => {
                    todo!("usb printer needs config file support for reading slicer config and nice trait for slice and print like network printer has")
                }
                MachineHandle::NetworkPrinter(printer) => {
                    let result = printer.client.slice_and_print(&file).await?;
                    println!("{:?}", result);
                }
            }
            Ok(())
        })
    }
}
