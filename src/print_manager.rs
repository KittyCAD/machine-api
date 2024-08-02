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
            machine.slice_and_print(&file).await?;

            Ok(())
        })
    }
}
