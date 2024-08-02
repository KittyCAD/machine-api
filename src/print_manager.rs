use std::path::PathBuf;

use anyhow::Result;

use crate::machine::{MachineHandle, Message};

pub struct PrintJob {
    pub file: PathBuf,
    pub machine: MachineHandle,
    pub job_name: String,
}

impl PrintJob {
    pub async fn spawn(&self) -> tokio::task::JoinHandle<Result<Message>> {
        let file = self.file.clone();
        let machine = self.machine.clone();
        let job_name = self.job_name.clone();
        tokio::task::spawn(async move {
            let result = machine.slice_and_print(&job_name, &file).await?;

            Ok(result)
        })
    }
}
