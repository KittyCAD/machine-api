use std::collections::HashMap;

use machine_api::{bambu as crate_bambu, moonraker as crate_moonraker, usb as crate_usb};
use serde::{Deserialize, Serialize};

mod bambu;
mod moonraker;
mod noop;
mod usb;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub machines: HashMap<String, MachineConfig>,
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
