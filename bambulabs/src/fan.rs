//! Fans in the printer.

use parse_display::{Display, FromStr};
use serde::{Deserialize, Serialize};

/// Enum for the fans in the printer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, FromStr)]
#[serde(rename_all = "snake_case")]
#[display(style = "snake_case")]
pub enum Fan {
    /// The part cooling fan.
    PartCooling = 1,
    /// The auxiliary fan.
    Auxiliary = 2,
    /// The chamber fan.
    Chamber = 3,
}
