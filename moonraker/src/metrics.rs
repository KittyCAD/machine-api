use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::Client;

/// Temperature readings from a heated element controlled by klipper.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ControlledTemperatureReadings {
    /// Observed temperatures, from oldest (0th) to latest (last)
    pub temperatures: Vec<f64>,

    /// Target temperatures, from oldest (0th) to latest (last)
    pub targets: Vec<f64>,

    /// Controlled power level, from oldest (0th) to latest (last)
    pub powers: Vec<f64>,
}

/// TemperatureReadings as reported by klipper.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TemperatureReadings {
    /// Information about the 3D printer extruder head.
    pub extruder: ControlledTemperatureReadings,

    /// Information about a heated bed, if present
    pub heater_bed: Option<ControlledTemperatureReadings>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct TemperatureReadingsWrapper {
    result: TemperatureReadings,
}

impl Client {
    /// Print an uploaded file.
    pub async fn temperatures(&self) -> Result<TemperatureReadings> {
        let client = reqwest::Client::new();

        let resp: TemperatureReadingsWrapper = client
            .get(format!("{}/server/temperature_store", self.url_base))
            .send()
            .await?
            .json()
            .await?;

        Ok(resp.result)
    }
}
