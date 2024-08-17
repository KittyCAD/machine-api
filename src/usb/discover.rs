use super::{Usb, UsbMachineInfo};
use crate::{
    Discover as DiscoverTrait, MachineMakeModel, MachineType, SimpleDiscovery as CrateSimpleDiscovery, Volume,
};
use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc::Sender, Mutex};
use tokio_serial::{SerialPortBuilderExt, SerialPortType};

/// Metadata about the Hardware in question.
///
/// MachineType of the hardware, Volume of the print bed, baud rate,
/// manufacturer, and model.
pub type UsbHardwareMetadata = (MachineType, Option<Volume>, u32, Option<String>, Option<String>);

type SimpleDiscovery = CrateSimpleDiscovery<(u16, u16, String), UsbHardwareMetadata, Usb>;

/// Handle to allow for USB based discovery of hardware.
#[derive(Clone)]
pub struct UsbDiscover {
    known_hardware: HashMap<(u16, u16, String), UsbHardwareMetadata>,
    discovery: Arc<Mutex<Option<SimpleDiscovery>>>,
}

impl UsbDiscover {
    /// Create a new USB discovery handler that searches for the already
    /// understood hardware.
    pub fn new(known_hardware: HashMap<(u16, u16, String), UsbHardwareMetadata>) -> Self {
        Self {
            known_hardware,
            discovery: Arc::new(Mutex::new(None)),
        }
    }
}

impl DiscoverTrait for UsbDiscover {
    type Error = anyhow::Error;
    type Control = Usb;

    async fn discover(&self, found: Sender<UsbMachineInfo>) -> Result<()> {
        let discovery = SimpleDiscovery::new(self.known_hardware.clone(), found);
        *self.discovery.lock().await = Some(discovery.clone());

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
                    tracing::trace!("skipping {:?}", port);
                    continue;
                };

                tracing::trace!("found a usb port");

                let serial = port_info.serial_number.unwrap_or("".to_string());
                let key = (port_info.vid, port_info.pid, serial.clone());

                if discovery.machine_info(&key).await.is_some() {
                    tracing::trace!(serial = serial, "found already known machine");
                    continue;
                }

                let Some(usb_metadata) = discovery.machine_config(&key).await else {
                    tracing::trace!(serial = serial, "machine not configured; skipping");
                    continue;
                };

                let (machine_type, machine_volume, serial_baud, machine_make, machine_model) = usb_metadata;
                tracing::debug!(port_name = port.port_name, serial = serial, "discovered new device");

                let make_model = MachineMakeModel {
                    manufacturer: machine_make.clone(),
                    model: machine_model.clone(),
                    serial: Some(serial.to_string()),
                };

                let stream = match tokio_serial::new(port.port_name.clone(), serial_baud).open_native_async() {
                    Err(e) => {
                        tracing::warn!(
                            port_name = port.port_name,
                            serial = serial,
                            error = format!("{:?}", e),
                            "failed to open USB device"
                        );
                        continue;
                    }
                    Ok(v) => v,
                };

                tracing::info!(port_name = port.port_name, serial = serial, "connected to new device");

                let machine_info = UsbMachineInfo::new(
                    machine_type,
                    make_model.clone(),
                    machine_volume,
                    port_info.vid,
                    port_info.pid,
                    port.port_name.clone(),
                    serial_baud,
                );

                let usb = Usb::new(stream, machine_info.clone());
                discovery.insert(key, machine_info, usb).await;
                tracing::trace!(port_name = port.port_name, serial = serial, "logged as discovered");
            }

            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }

    async fn connect(&self, machine: UsbMachineInfo) -> Result<Arc<Mutex<Usb>>> {
        let discovery = {
            let discovery = self.discovery.lock().await;
            let Some(discovery) = discovery.as_ref() else {
                anyhow::bail!("UsbDiscover::discover not called yet");
            };
            discovery.clone()
        };
        let key = machine.discovery_key();

        let Some(machine) = discovery.machine(&key).await else {
            anyhow::bail!("unknown machine");
        };
        Ok(machine)
    }
}
