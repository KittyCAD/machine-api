use crate::{
    Control as ControlTrait, ControlGcode as ControlGcodeTrait, ControlSuspend as ControlSuspendTrait, TemporaryFile,
    Volume,
};
use anyhow::Result;
use bambulabs::{client::Client, command::Command};
use std::sync::Arc;

/// Control channel handle to a Bambu Labs X1 Carbon.
#[derive(Clone)]
pub struct X1Carbon {
    client: Arc<Client>,
}

impl X1Carbon {
    /// Return a borrow of the underlying Client.
    pub fn inner(&self) -> &Client {
        self.client.as_ref()
    }

    /// Get the latest status of the printer.
    pub fn get_status(&self) -> Result<Option<bambulabs::message::PushStatus>> {
        self.client.get_status()
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

    async fn max_part_volume(&self) -> Result<Volume> {
        Ok(Volume {
            width: 256.0,
            height: 256.0,
            depth: 256.0,
        })
    }

    async fn emergency_stop(&self) -> Result<()> {
        self.stop().await
    }

    async fn stop(&self) -> Result<()> {
        self.client.publish(Command::stop()).await?;
        Ok(())
    }
}

impl ControlSuspendTrait for X1Carbon {
    async fn pause(&self) -> Result<()> {
        self.client.publish(Command::pause()).await?;
        Ok(())
    }

    async fn resume(&self) -> Result<()> {
        self.client.publish(Command::resume()).await?;
        Ok(())
    }
}

impl ControlGcodeTrait for X1Carbon {
    async fn build(&self, job_name: &str, gcode: TemporaryFile) -> Result<()> {
        // Upload the file to the printer.
        self.client.upload_file(gcode.path()).await?;

        // Get just the filename.
        let filename = gcode
            .path()
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("No filename: {}", gcode.path().display()))?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Bad filename: {}", gcode.path().display()))?;

        // Check if the printer has an AMS.
        let has_ams = self.has_ams()?;

        self.client
            .publish(Command::print_file(job_name, filename, has_ams))
            .await?;

        Ok(())
    }
}
