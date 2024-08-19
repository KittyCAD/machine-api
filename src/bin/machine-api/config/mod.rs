use machine_api::usb as crate_usb;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod usb;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub machines: HashMap<String, MachineConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum MachineConfig {
    Usb(crate_usb::Config),
    Noop {},
}
