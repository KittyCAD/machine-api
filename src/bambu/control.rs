use anyhow::Result;
use bambulabs::{client::Client, command::Command};

use super::{PrinterInfo, X1Carbon};
use crate::{
    Control as ControlTrait, MachineInfo as MachineInfoTrait, MachineMakeModel, MachineState, MachineType,
    SuspendControl as SuspendControlTrait, ThreeMfControl as ThreeMfControlTrait, ThreeMfTemporaryFile, Volume,
};

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

impl MachineInfoTrait for PrinterInfo {
    fn machine_type(&self) -> MachineType {
        MachineType::FusedDeposition
    }

    fn make_model(&self) -> MachineMakeModel {
        self.make_model.clone()
    }

    fn max_part_volume(&self) -> Option<Volume> {
        Some(Volume {
            width: 256.0,
            height: 256.0,
            depth: 256.0,
        })
    }
}
impl ControlTrait for X1Carbon {
    type Error = anyhow::Error;
    type MachineInfo = PrinterInfo;

    async fn machine_info(&self) -> Result<PrinterInfo> {
        Ok(self.info.clone())
    }

    async fn emergency_stop(&mut self) -> Result<()> {
        self.stop().await
    }

    async fn stop(&mut self) -> Result<()> {
        self.client.publish(Command::stop()).await?;
        Ok(())
    }

    async fn healthy(&self) -> bool {
        // TODO: fix this
        true
    }

    async fn state(&self) -> Result<MachineState> {
        let Some(status) = self.client.get_status()? else {
            return Ok(MachineState::Unknown);
        };

        let Some(state) = status.gcode_state else {
            return Ok(MachineState::Unknown);
        };

        let more_string = status.stg_cur.map(|s| s.to_string());

        match state {
            bambulabs::message::GcodeState::Idle | bambulabs::message::GcodeState::Finish => Ok(MachineState::Idle),
            bambulabs::message::GcodeState::Running => Ok(MachineState::Running),
            bambulabs::message::GcodeState::Pause => Ok(MachineState::Paused),
            bambulabs::message::GcodeState::Failed => Ok(MachineState::Failed(more_string)),
        }
    }
}

impl SuspendControlTrait for X1Carbon {
    async fn pause(&mut self) -> Result<()> {
        self.client.publish(Command::pause()).await?;
        Ok(())
    }

    async fn resume(&mut self) -> Result<()> {
        self.client.publish(Command::resume()).await?;
        Ok(())
    }
}

impl ThreeMfControlTrait for X1Carbon {
    async fn build(&mut self, job_name: &str, gcode: ThreeMfTemporaryFile) -> Result<()> {
        let gcode = gcode.0;

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
