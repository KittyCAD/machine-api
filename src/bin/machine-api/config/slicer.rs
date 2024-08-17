// use anyhow::Result;
// use machine_api::slicer::{orca, prusa, AnySlicer};
use serde::{Deserialize, Serialize};
// use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum SlicerConfig {
    /// Use the Prusa Slicer.
    Prusa { config: String },

    /// Use the Orca Slicer.
    Orca { config: String },
}

impl SlicerConfig {
    // pub async fn load(&self) -> Result<AnySlicer> {
    //     Ok(match self {
    //         Self::Prusa { config } => {
    //             let path: PathBuf = config.parse().unwrap();
    //             prusa::Slicer::new(&path).into()
    //         }
    //         Self::Orca { config } => {
    //             let path: PathBuf = config.parse().unwrap();
    //             orca::Slicer::new(&path).into()
    //         }
    //     })
    // }
}
