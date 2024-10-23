use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_serial::{SerialPortBuilderExt, SerialPortType};

use super::UsbVariant;
use crate::{slicer, usb, Discover, Filament, Machine, MachineMakeModel};

/// Configuration block for a USB based device.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    /// Slicer configuration for created Machine.
    pub slicer: slicer::Config,

    /// Information regarding the specific make/model of device.
    pub variant: UsbVariant,

    /// Baud rate to use when opening the serial pty.
    pub baud: Option<u32>,

    /// Serial number, as reported by the USB protocol. None will match
    /// any USB device.
    pub serial: Option<String>,

    /// USB Vendor ID (vid) to scan for. None will match any USB device.
    pub vendor_id: Option<u16>,

    /// USB Product ID (pid) to scan for. None will match any USB device.
    pub product_id: Option<u16>,

    /// Extrusion hotend nozzle's diameter.
    pub nozzle_diameter: f64,

    /// Available filaments.
    pub filaments: Vec<Filament>,

    /// Currently loaded filament, if possible to determine.
    pub loaded_filament_idx: Option<usize>,
}

impl Config {
    fn get_baud(&self) -> u32 {
        self.baud.unwrap_or(self.variant.get_baud().unwrap_or(115200))
    }

    /// check to see if this qualifies as a match
    fn matches(&self, found: &SerialPort) -> bool {
        let (vid, pid, serial) = found;

        if *vid != self.vendor_id.unwrap_or(*vid) {
            tracing::trace!(vid = vid, config_vid = self.vendor_id, "vendor_id does not match");
            return false;
        }

        if *pid != self.product_id.unwrap_or(*pid) {
            tracing::trace!(pid = pid, config_pid = self.product_id, "product_id does not match");
            return false;
        }

        if *serial != self.serial {
            tracing::trace!(serial = serial, config_serial = self.serial, "serial does not match");
            return false;
        }

        true
    }
}

/// USB Discovery system -- scan for any attached USB devices through the
/// Discovery trait.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UsbDiscovery {
    /// known devices to the discovery routine
    configs: HashMap<String, Config>,
}

impl UsbDiscovery {
    /// Create a new USB Discovery scanner.
    pub fn new<ConfigsT: Into<HashMap<String, Config>>>(cfgs: ConfigsT) -> Self {
        Self { configs: cfgs.into() }
    }

    /// Attempt to match the SerialPort to a known config block.
    async fn find_match(&self, port: &SerialPort) -> Option<(String, Config)> {
        for (machine_id, configuration) in self.configs.iter() {
            tracing::trace!(
                vid = port.0,
                pid = port.1,
                serial = port.2,
                machine_id = machine_id,
                config_vid = configuration.vendor_id,
                config_pid = configuration.product_id,
                config_serial = configuration.serial,
                "checking to see if device matches config"
            );
            if configuration.matches(port) {
                tracing::trace!(
                    vid = port.0,
                    pid = port.1,
                    serial = port.2,
                    machine_id = machine_id,
                    "match found",
                );
                return Some((machine_id.clone(), configuration.clone()));
            }
        }
        tracing::trace!(
            vid = port.0,
            pid = port.1,
            serial = port.2,
            "no matching config blocks found",
        );
        None
    }
}

type SerialPort = (u16, u16, Option<String>);

impl Discover for UsbDiscovery {
    type Error = anyhow::Error;

    async fn discover(
        &self,
        channel: tokio::sync::mpsc::Sender<String>,
        found: Arc<RwLock<HashMap<String, RwLock<Machine>>>>,
    ) -> Result<()> {
        if self.configs.is_empty() {
            tracing::debug!("no usb devices configured, shutting down usb scans");
            return Ok(());
        }

        loop {
            tracing::debug!("scanning serial ports");
            let ports = match tokio_serial::available_ports() {
                Err(e) => {
                    tracing::warn!(error = format!("{:?}", e), "can not enumerate usb devices");
                    continue;
                }
                Ok(v) => v,
            };

            for port in ports {
                let SerialPortType::UsbPort(port_info) = port.port_type else {
                    tracing::trace!("skipping {:?}; not a USB port", port);
                    continue;
                };

                let port_name = port.port_name.clone();
                let port: SerialPort = (port_info.vid, port_info.pid, port_info.serial_number.clone());

                tracing::trace!(
                    vid = port.0,
                    pid = port.1,
                    serial = port.2,
                    "found a usb port, checking for matches"
                );

                let Some((machine_id, config)) = self.find_match(&port).await else {
                    tracing::trace!(vid = port.0, pid = port.1, serial = port.2, "no matches, moving on",);
                    continue;
                };

                tracing::trace!(
                    machine_id = machine_id,
                    vid = port.0,
                    pid = port.1,
                    serial = port.2,
                    "found a matching config; checking if known"
                );

                if found.read().await.get(&machine_id).is_some() {
                    tracing::trace!(machine_id = machine_id, "machine already exists, skipping",);
                    continue;
                }

                tracing::info!(
                    machine_id = machine_id,
                    vid = port.0,
                    pid = port.1,
                    serial = port.2,
                    "found a new usb connected machine"
                );

                let baud = config.get_baud();

                let stream = match tokio_serial::new(port_name.clone(), baud).open_native_async() {
                    Err(e) => {
                        tracing::warn!(
                            machine_id = machine_id,
                            vid = port.0,
                            pid = port.1,
                            serial = port.2,
                            port_name = port_name,
                            error = format!("{:?}", e),
                            "failed to open USB device"
                        );
                        continue;
                    }
                    Ok(v) => v,
                };

                let (manufacturer, model) = config.variant.get_manufacturer_model();

                let slicer = config.slicer.load()?;

                found.write().await.insert(
                    machine_id.clone(),
                    RwLock::new(Machine::new(
                        usb::Usb::new(
                            stream,
                            usb::UsbMachineInfo::new(
                                config.variant.get_machine_type(),
                                MachineMakeModel {
                                    manufacturer,
                                    model,
                                    serial: port.2,
                                },
                                config.variant.get_max_part_volume(),
                                port.0,
                                port.1,
                                port_name.clone(),
                                baud,
                            ),
                            config.clone(),
                        ),
                        slicer,
                    )),
                );
                let _ = channel.send(machine_id).await;
            }

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }
}
