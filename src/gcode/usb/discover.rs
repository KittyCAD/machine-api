use super::{Usb, UsbMachineInfo};
use crate::{Control as ControlTrait, Discover as DiscoverTrait}; // MachineMakeModel, MachineType, Volume};
use anyhow::Result;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tokio_serial::SerialPortType; // , SerialStream, UsbPortInfo};

/// A specific known bit of hardware that is USB connected.
///
/// Order of arguments here is the VID (Vendor ID), PID (Product ID), and then
/// the specific method to take that information and turn it into a control
/// channel.
pub type UsbKnownDevice = (
    u16,
    u16,
    u32,
    // MachineType,
    // Option<Volume>,
    // // Product
    // (&'a str, &'a str),
);

/// Library of known devices.
pub struct UsbKnownDevices(Vec<UsbKnownDevice>);

/// Handle to allow for USB based discovery of hardware.
pub struct UsbDiscover {
    devices: Arc<Mutex<HashMap<String, Usb>>>,
}

impl DiscoverTrait for UsbDiscover {
    type Error = anyhow::Error;
    type MachineInfo = UsbMachineInfo;
    type Control = Usb;

    async fn discover(&self) -> Result<()> {
        let devices = self.devices.clone();
        tokio::spawn(async {
            loop {
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

                    // check
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
