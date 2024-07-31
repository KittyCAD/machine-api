//! Features on the printer.

use parse_display::{Display, FromStr};
use serde::{Deserialize, Serialize};

/// Enum for the features on the printer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, FromStr)]
#[serde(rename_all = "snake_case")]
#[display(style = "snake_case")]
pub enum Features {
    /// The auxiliary fan.
    AuxFan = 1,
    /// The chamber light.
    ChamberLight = 2,
    /// The chamber fan.
    ChamberFan = 3,
    /// The chamber temperature.
    ChamberTemperature = 4,
    /// The current stage.
    CurrentStage = 5,
    /// The print layers.
    PrintLayers = 6,
    /// The ams.
    Ams = 7,
    /// The external spool.
    ExternalSpool = 8,
    /// The k value.
    KValue = 9,
    /// The start time.
    StartTime = 10,
    /// The ams temperature.
    AmsTemperature = 11,
    /// The camera.
    CameraRtsp = 13,
    /// The start time generated.
    StartTimeGenerated = 14,
    /// The camera image.
    CameraImage = 15,
}
