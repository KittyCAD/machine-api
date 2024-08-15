use super::{Usb, UsbMachineInfo};
use crate::{Control as ControlTrait, Discover as DiscoverTrait};
use crate::{MachineMakeModel, MachineType, Volume};
use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tokio_serial::{SerialPortBuilderExt, SerialPortType};

/// Metadata about the Hardware in question.
///
/// MachineType of the hardware, Volume of the print bed, baud rate,
/// manufacturer, and model.
type UsbHardwareMetadata = (MachineType, Option<Volume>, u32, Option<String>, Option<String>);

/// Handle to allow for USB based discovery of hardware.
pub struct UsbDiscover {
    devices: Arc<Mutex<HashMap<(u16, u16, String), Usb>>>,
    known_hardware: HashMap<(u16, u16, String), UsbHardwareMetadata>,
}

impl UsbDiscover {
    /// Create a new USB discovery handler that searches for the already
    /// understood hardware.
    pub fn new(known_hardware: HashMap<(u16, u16, String), UsbHardwareMetadata>) -> Self {
        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
            known_hardware,
        }
    }
}

impl DiscoverTrait for UsbDiscover {
    type Error = anyhow::Error;
    type MachineInfo = UsbMachineInfo;
    type Control = Usb;

    async fn discover(&self) -> Result<()> {
        let devices = self.devices.clone();
        let known_hardware = self.known_hardware.clone();
        tokio::spawn(async move {
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
                        continue;
                    };
                    let Some(serial) = port_info.serial_number else {
                        tracing::trace!(port_name = port.port_name, "no serial reported");
                        continue;
                    };

                    let usb_id = (port_info.vid, port_info.pid, serial.clone());

                    if devices.lock().await.get(&usb_id).is_some() {
                        tracing::trace!(serial = serial, "found already known machine");
                        continue;
                    }

                    let Some(usb_metadata) = known_hardware.get(&usb_id) else {
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

                    let stream = match tokio_serial::new(port.port_name.clone(), *serial_baud).open_native_async() {
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
                        *machine_type,
                        make_model.clone(),
                        *machine_volume,
                        port_info.vid,
                        port_info.pid,
                        port.port_name.clone(),
                        *serial_baud,
                    );

                    devices.lock().await.insert(usb_id, Usb::new(stream, machine_info));
                }

                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
        Ok(())
    }

    async fn discovered(&self) -> Result<Vec<UsbMachineInfo>> {
        let mut machines = vec![];
        for (_, machine) in self.devices.lock().await.iter() {
            machines.push(machine.machine_info().await?);
        }
        Ok(machines)
    }

    async fn connect(&self, _machine: UsbMachineInfo) -> Result<Arc<Mutex<Usb>>> {
        unimplemented!()
    }
}
