use anyhow::Result;
use std::path::Path;

use super::PrintManager;

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

    /// Cancel a print job.
    pub async fn cancel_print(&self) -> Result<()> {
        let client = reqwest::Client::new();
        client
            .post(format!("{}/printer/print/cancel", self.url_base))
            .send()
            .await?;
        Ok(())
    }
}
