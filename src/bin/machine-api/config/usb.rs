use super::{Config, MachineConfig, SlicerConfig};
use anyhow::Result;
use machine_api::{
    usb::{UsbDiscover, UsbHardwareMetadata},
    MachineType, Volume,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Specific make/model of device we're connected to.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum UsbVariant {
    /// Prusa Labs MK3.
    PrusaMk3,

    /// Generic gcode-based printer.
    Generic,
}

impl UsbVariant {
    /// Get the Vendor ID and the Product ID of the USB variant.
    pub fn get_vid_pid(&self) -> (Option<u16>, Option<u16>) {
        match self {
            UsbVariant::PrusaMk3 => (Some(0x2c99), Some(0x0002)),
            UsbVariant::Generic => (None, None),
        }
    }

    /// Return the [MachineType] of the variant.
    pub fn machine_type(&self) -> MachineType {
        match self {
            UsbVariant::PrusaMk3 => MachineType::FusedDeposition,
            UsbVariant::Generic => MachineType::FusedDeposition,
        }
    }

    /// Return the max part volume.
    pub fn max_part_volume(&self) -> Option<Volume> {
        match self {
            UsbVariant::PrusaMk3 => None,
            UsbVariant::Generic => None,
        }
    }

    /// baud rate of the machine
    pub fn baud(&self) -> Option<u32> {
        match self {
            UsbVariant::PrusaMk3 => Some(115200),
            UsbVariant::Generic => None,
        }
    }

    /// Return the make and model.
    pub fn manufacturer_model(&self) -> (Option<String>, Option<String>) {
        match self {
            UsbVariant::PrusaMk3 => (Some("Prusa Research".to_string()), Some("MK3".to_string())),
            UsbVariant::Generic => (None, None),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MachineConfigUsb {
    slicer: SlicerConfig,
    variant: UsbVariant,
    serial: String,
    baud: Option<u32>,
    vendor_id: Option<u16>,
    product_id: Option<u16>,
}

impl MachineConfigUsb {
    pub fn get_key(&self) -> (u16, u16, String) {
        let (vid, pid) = self.get_vid_pid();
        (vid, pid, self.serial.clone())
    }

    pub fn get_vid_pid(&self) -> (u16, u16) {
        let (vid, pid) = self.variant.get_vid_pid();
        let vid = self.vendor_id.unwrap_or(vid.unwrap_or(0));
        let pid = self.product_id.unwrap_or(pid.unwrap_or(0));
        (vid, pid)
    }

    pub fn get_baud(&self) -> u32 {
        self.baud.unwrap_or(self.variant.baud().unwrap_or(0))
    }

    pub fn check(&self) -> Result<()> {
        let (pid, vid) = self.get_vid_pid();
        let baud = self.get_baud();

        if pid == 0 {
            anyhow::bail!("no usb product id (pid) set, and the variant didn't have one");
        }
        if vid == 0 {
            anyhow::bail!("no usb vendor id (vid) set, and the variant didn't have one");
        }
        if baud == 0 {
            anyhow::bail!("no baud rate set, and the variant didn't have one");
        }
        Ok(())
    }

    pub fn usb_hardware_metadata(&self) -> Result<UsbHardwareMetadata> {
        let volume = self.variant.max_part_volume();
        let (manufacturer, model) = self.variant.manufacturer_model();
        let baud = self.get_baud();

        Ok((self.variant.machine_type(), volume, baud, manufacturer, model))
    }
}

impl Config {
    /// Load a UsbDiscover stub based on the provided machine config.
    pub async fn load_discover_usb(&self) -> Result<Option<UsbDiscover>> {
        let mut known_devices = HashMap::new();
        for (_machine_key, machine) in self.machines.iter().filter_map(|(machine_key, machine)| {
            if let MachineConfig::Usb(usb) = machine {
                Some((machine_key, usb))
            } else {
                None
            }
        }) {
            machine.check()?;
            known_devices.insert(machine.get_key(), machine.usb_hardware_metadata()?);
        }

        if !known_devices.is_empty() {
            Ok(Some(UsbDiscover::new(known_devices)))
        } else {
            Ok(None)
        }
    }
}
