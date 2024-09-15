use machine_api::{bambu as crate_bambu, moonraker as crate_moonraker, usb as crate_usb};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

mod bambu;
mod moonraker;
mod noop;
mod usb;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Config {
    pub server: Option<GlobalConfig>,

    pub machines: HashMap<String, MachineConfig>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GlobalConfig {
    /// Program to run on every shapefile before sending it off to the
    /// printer, given the STL and GCode.
    ///
    /// hook stl gcode
    ///
    /// Any code other than 0 will result in an aborted print.
    pub hook: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum MachineConfig {
    Usb(crate_usb::Config),
    Noop {},
    Moonraker(crate_moonraker::Config),
    Bambu(crate_bambu::Config),
}
