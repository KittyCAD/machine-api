use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::PrintManager;

/// Information about the underlying Klipper runtime and host computer.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InfoResponse {
    /// slug defining what state the printer is currently in, such as
    /// `ready`.
    pub state: String,

    /// Human readable description of the state above.
    pub state_message: String,

    /// Hostname of the host computer.
    pub hostname: String,

    /// Version of Klipper running on the host computer.
    pub software_version: String,

    /// CPU of the host running Klipper.
    pub cpu_info: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct InfoResponseWrapper {
    pub result: InfoResponse,
}

impl PrintManager {
    /// Print an uploaded file.
    pub async fn print(&self, file_name: &Path) -> Result<()> {
        let file_name = file_name.to_str().unwrap();
        let client = reqwest::Client::new();
        client
            .post(format!("{}/printer/print/start", self.url_base))
            .form(&[("filename", file_name)])
            .send()
            .await?;
        Ok(())
    }

    /// This endpoint will immediately halt the printer and put it in a
    /// "shutdown" state. It should be used to implement an "emergency stop"
    /// button and also used if a user enters M112(emergency stop) via a
    /// console.
    pub async fn emergency_stop(&self) -> Result<()> {
        let client = reqwest::Client::new();
        client
            .post(format!("{}/printer/emergency_stop", self.url_base))
            .send()
            .await?;
        Ok(())
    }

    /// Get information regarding the processor and its state.
    pub async fn info(&self) -> Result<InfoResponse> {
        let client = reqwest::Client::new();
        let resp: InfoResponseWrapper = client
            .post(format!("{}/printer/info", self.url_base))
            .send()
            .await?
            .json()
            .await?;
        Ok(resp.result)
    }

    /// Restart the printer (shut down and reboot).
    pub async fn restart(&self) -> Result<()> {
        let client = reqwest::Client::new();
        client.post(format!("{}/printer/restart", self.url_base)).send().await?;
        Ok(())
    }

    /// Cancel a print job.
    pub async fn cancel_print(&self) -> Result<()> {
        let client = reqwest::Client::new();
        client
            .post(format!("{}/printer/print/cancel", self.url_base))
            .send()
            .await?;
        Ok(())
    }

    /// Pause a print job.
    pub async fn pause_print(&self) -> Result<()> {
        let client = reqwest::Client::new();
        client
            .post(format!("{}/printer/print/pause", self.url_base))
            .send()
            .await?;
        Ok(())
    }

    /// Resume a print job.
    pub async fn resume_print(&self) -> Result<()> {
        let client = reqwest::Client::new();
        client
            .post(format!("{}/printer/print/resume", self.url_base))
            .send()
            .await?;
        Ok(())
    }
}
