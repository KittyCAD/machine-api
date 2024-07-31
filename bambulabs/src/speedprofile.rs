//! Speed profiles for the Bambu printers.

use parse_display::{Display, FromStr};
use serde::{Deserialize, Serialize};

/// Speed profiles for the Bambu printers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, FromStr, Serialize, Deserialize)]
#[display(style = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SpeedProfile {
    /// Silent mode.
    Silent,
    /// Standard mode.
    Standard,
    /// Sport mode.
    Sport,
    /// Ludicrous mode.
    Ludicrous,
}
