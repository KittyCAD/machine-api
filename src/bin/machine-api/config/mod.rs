use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod moonraker;
mod slicer;
mod usb;

use slicer::SlicerConfig;
// use usb::{MachineConfigUsb, UsbVariant};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub machines: HashMap<String, MachineConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum MachineConfig {
    Usb(usb::MachineConfigUsb),
    Noop {},
    Moonraker(moonraker::MachineConfigMoonraker),
    Bambu {},
}

impl Config {}
