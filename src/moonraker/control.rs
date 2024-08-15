use super::Client;
use crate::{
    Control as ControlTrait, GcodeControl as GcodeControlTrait, GcodeTemporaryFile, MachineInfo as MachineInfoTrait,
    MachineMakeModel, MachineType, SuspendControl as SuspendControlTrait, Volume,
};
use anyhow::Result;
use moonraker::InfoResponse;
use std::path::PathBuf;

/// Information about the connected Moonraker-based printer.
pub struct MachineInfo {
    inner: InfoResponse,
    make_model: MachineMakeModel,
    volume: Option<Volume>,
}

impl MachineInfoTrait for MachineInfo {
    fn machine_type(&self) -> MachineType {
        MachineType::FusedDeposition
    }

    fn make_model(&self) -> MachineMakeModel {
        self.make_model.clone()
    }

    fn max_part_volume(&self) -> Option<Volume> {
        self.volume
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
        tracing::debug!("machine_info called");
        Ok(MachineInfo {
            inner: self.client.info().await?,
            make_model: self.make_model.clone(),
            volume: self.volume,
        })
    }

    async fn emergency_stop(&mut self) -> Result<()> {
        tracing::warn!("emergency stop requested");
        self.client.emergency_stop().await
    }

    async fn stop(&mut self) -> Result<()> {
        tracing::debug!("stop requested");
        self.client.cancel_print().await
    }
}

impl SuspendControlTrait for Client {
    async fn pause(&mut self) -> Result<()> {
        tracing::debug!("pause requested");
        self.client.pause_print().await
    }

    async fn resume(&mut self) -> Result<()> {
        tracing::debug!("resume requested");
        self.client.resume_print().await
    }
}

impl GcodeControlTrait for Client {
    async fn build(&mut self, job_name: &str, gcode: GcodeTemporaryFile) -> Result<()> {
        let gcode = gcode.0;

        tracing::info!(job_name = job_name, "uploading and printing gcode");
        tracing::debug!("uploading");
        let path: PathBuf = self.client.upload_file(gcode.path()).await?.item.path.parse()?;
        tracing::debug!("printing");
        self.client.print(&path).await?;
        Ok(())
    }
}