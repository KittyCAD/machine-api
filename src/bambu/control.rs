use super::{PrinterInfo, X1Carbon};
use crate::{
    Control as ControlTrait, SuspendControl as SuspendControlTrait, ThreeMfControl as ThreeMfControlTrait,
    ThreeMfTemporaryFile,
};
use anyhow::Result;
use bambulabs::{client::Client, command::Command};
use std::sync::Arc;

impl X1Carbon {
    /// Get the client.
    fn get_client(&self) -> Result<Arc<Client>> {
        let entry = self
            .discover
            .printers
            .get(&self.name)
            .ok_or_else(|| anyhow::anyhow!("printer has not been discovered yet"))?;
        Ok(entry.value().0.clone())
    }
    /// Get the PrinterInfo
    fn get_printer_info(&self) -> Result<PrinterInfo> {
        let entry = self
            .discover
            .printers
            .get(&self.name)
            .ok_or_else(|| anyhow::anyhow!("printer has not been discovered yet"))?;
        Ok(entry.value().1.clone())
    }

    /// Get the latest status of the printer.
    pub fn get_status(&self) -> Result<Option<bambulabs::message::PushStatus>> {
        self.get_client()?.get_status()
    }

    /// Check if the printer has an AMS.
    pub fn has_ams(&self) -> Result<bool> {
        let Some(status) = self.get_status()? else {
            return Ok(false);
        };

        let Some(ams) = status.ams else {
            return Ok(false);
        };

        let Some(ams_exists) = ams.ams_exist_bits else {
            return Ok(false);
        };

        Ok(ams_exists != "0")
    }
}

impl ControlTrait for X1Carbon {
    type Error = anyhow::Error;
    type MachineInfo = PrinterInfo;

    async fn machine_info(&self) -> Result<PrinterInfo> {
        self.get_printer_info()
    }

    async fn emergency_stop(&mut self) -> Result<()> {
        self.stop().await
    }

    async fn stop(&mut self) -> Result<()> {
        self.get_client()?.publish(Command::stop()).await?;
        Ok(())
    }
}

impl SuspendControlTrait for X1Carbon {
    async fn pause(&mut self) -> Result<()> {
        self.get_client()?.publish(Command::pause()).await?;
        Ok(())
    }

    async fn resume(&mut self) -> Result<()> {
        self.get_client()?.publish(Command::resume()).await?;
        Ok(())
    }
}

impl ThreeMfControlTrait for X1Carbon {
    async fn build(&mut self, job_name: &str, gcode: ThreeMfTemporaryFile) -> Result<()> {
        let gcode = gcode.0;

        // Upload the file to the printer.
        self.get_client()?.upload_file(gcode.path()).await?;

        // Get just the filename.
        let filename = gcode
            .path()
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("No filename: {}", gcode.path().display()))?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Bad filename: {}", gcode.path().display()))?;

        // Check if the printer has an AMS.
        let has_ams = self.has_ams()?;

        self.get_client()?
            .publish(Command::print_file(job_name, filename, has_ams))
            .await?;

        Ok(())
    }
}
