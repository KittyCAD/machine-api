use super::Client;
use crate::{
    Control as ControlTrait, ControlGcode as ControlGcodeTrait, ControlSuspend as ControlSuspendTrait,
    MachineInfo as MachineInfoTrait, MachineMakeModel, MachineType, TemporaryFile, Volume,
};
use anyhow::Result;
use moonraker::InfoResponse;
use std::path::PathBuf;

/// Information about the connected Moonraker-based printer.
pub struct MachineInfo {
    inner: InfoResponse,
    volume: Volume,
}

impl MachineInfoTrait for MachineInfo {
    type Error = anyhow::Error;

    fn machine_type(&self) -> MachineType {
        MachineType::FusedDeposition
    }

    fn make_model(&self) -> MachineMakeModel {
        // TODO: fix this
        MachineMakeModel {
            manufacturer: None,
            model: None,
            serial: None,
        }
    }

    fn max_part_volume(&self) -> Result<Volume> {
        Ok(self.volume.clone())
    }
}

impl MachineInfo {
    /// Return the raw response from the Moonraker API.
    pub fn inner(&self) -> &InfoResponse {
        &self.inner
    }
}

impl ControlTrait for Client {
    type Error = anyhow::Error;
    type MachineInfo = MachineInfo;

    async fn machine_info(&self) -> Result<MachineInfo> {
        Ok(MachineInfo {
            inner: self.client.info().await?,
            volume: self.volume.clone(),
        })
    }

    async fn emergency_stop(&self) -> Result<()> {
        self.client.emergency_stop().await
    }

    async fn stop(&self) -> Result<()> {
        self.client.cancel_print().await
    }
}

impl ControlSuspendTrait for Client {
    async fn pause(&self) -> Result<()> {
        self.client.pause_print().await
    }

    async fn resume(&self) -> Result<()> {
        self.client.resume_print().await
    }
}

impl ControlGcodeTrait for Client {
    async fn build(&self, _job_name: &str, gcode: TemporaryFile) -> Result<()> {
        let path: PathBuf = self.client.upload_file(gcode.path()).await?.item.path.parse()?;
        self.client.print(&path).await?;
        Ok(())
    }
}