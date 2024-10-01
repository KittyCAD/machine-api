use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{orca, prusa, AnySlicer};

/// Standard slicer config -- as used by the machine-api server and any
/// other consumers.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Config {
    /// Use the Prusa Slicer.
    Prusa {
        /// Use the provided `.ini` Slicer config.
        config: String,
    },

    /// Use the Orca Slicer.
    Orca {
        /// Use the provided `.ini` Slicer config.
        config: String,
    },
}

impl Config {
    /// Create a new Slicer from the provided configuration.
    pub fn load(&self) -> Result<AnySlicer> {
        Ok(match self {
            Self::Prusa { config } => {
                let path: PathBuf = config.parse()?;
                let path = std::fs::canonicalize(&path)?;
                prusa::Slicer::new(&path).into()
            }
            Self::Orca { config } => {
                let path: PathBuf = config.parse()?;
                let path = std::fs::canonicalize(&path)?;
                orca::Slicer::new(&path).into()
            }
        })
    }
}
