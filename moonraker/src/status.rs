use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::Client;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct VirtualSdcard {
    pub progress: f64,
    pub file_position: usize,
    pub is_active: bool,
    pub file_path: String,
    pub file_size: usize,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Webhooks {
    pub state: String,
    pub state_message: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PrintStats {
    pub print_duration: f64,
    pub total_duration: f64,
    pub filament_used: f64,
    pub filename: String,
    pub state: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Status {
    pub virtual_sdcard: VirtualSdcard,
    pub webhooks: Webhooks,
    pub print_stats: PrintStats,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
struct QueryResponse {
    status: Status,
    eventtime: f64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
struct QueryResponseWrapper {
    result: QueryResponse,
}

impl Client {
    /// Print an uploaded file.
    pub async fn status(&self) -> Result<Status> {
        tracing::debug!(base = self.url_base, "requesting status");
        let client = reqwest::Client::new();

        let resp: QueryResponseWrapper = client
            .get(format!(
                "{}/printer/objects/query?webhooks&virtual_sdcard&print_stats",
                self.url_base
            ))
            .send()
            .await?
            .json()
            .await?;

        Ok(resp.result.status)
    }
}
