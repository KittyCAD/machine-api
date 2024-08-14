use crate::{MachineInfo, MachineMakeModel, MachineType, Volume};
use anyhow::Result;

/// AnyMachine is any supported machine.
pub enum AnyMachine {
    /// Bambu Labs X1 Carbon
    #[cfg(feature = "bambu")]
    BambuX1Carbon(crate::bambu::X1Carbon),

    /// Generic Moonraker type printer
    #[cfg(feature = "moonraker")]
    Moonraker(crate::moonraker::Client),

    /// Generic USB-based gcode printer
    #[cfg(feature = "serial")]
    Usb(crate::gcode::Usb),
}

/// AnyMachineInfo is any supported machine's MachineInfo.
pub enum AnyMachineInfo {
    /// Bambu Labs X1 Carbon
    #[cfg(feature = "bambu")]
    BambuX1Carbon(crate::bambu::PrinterInfo),

    /// Generic Moonraker type printer
    #[cfg(feature = "moonraker")]
    Moonraker(crate::moonraker::MachineInfo),

    /// Generic USB-based gcode printer
    #[cfg(feature = "serial")]
    Usb(crate::gcode::UsbMachineInfo),
}

#[cfg(feature = "bambu")]
mod _bambu {
    use super::*;

    impl From<crate::bambu::X1Carbon> for AnyMachine {
        fn from(client: crate::bambu::X1Carbon) -> Self {
            Self::BambuX1Carbon(client)
        }
    }

    impl From<crate::bambu::PrinterInfo> for AnyMachineInfo {
        fn from(info: crate::bambu::PrinterInfo) -> Self {
            Self::BambuX1Carbon(info)
        }
    }
}

#[cfg(feature = "moonraker")]
mod _moonraker {
    use super::*;

    impl From<crate::moonraker::Client> for AnyMachine {
        fn from(client: crate::moonraker::Client) -> Self {
            Self::Moonraker(client)
        }
    }

    impl From<crate::moonraker::MachineInfo> for AnyMachineInfo {
        fn from(info: crate::moonraker::MachineInfo) -> Self {
            Self::Moonraker(info)
        }
    }
}

#[cfg(feature = "serial")]
mod _serial {
    use super::*;

    impl From<crate::gcode::Usb> for AnyMachine {
        fn from(client: crate::gcode::Usb) -> Self {
            Self::Usb(client)
        }
    }

    impl From<crate::gcode::UsbMachineInfo> for AnyMachineInfo {
        fn from(info: crate::gcode::UsbMachineInfo) -> Self {
            Self::Usb(info)
        }
    }
}

macro_rules! for_all {
    (|$slf:ident, $machine:ident| $body:block) => {
        match $slf {
            #[cfg(feature = "bambu")]
            Self::BambuX1Carbon($machine) => $body,

            #[cfg(feature = "moonraker")]
            Self::Moonraker($machine) => $body,

            #[cfg(feature = "serial")]
            Self::Usb($machine) => $body,
        }
    };
}

impl MachineInfo for AnyMachineInfo {
    fn machine_type(&self) -> MachineType {
        for_all!(|self, machine| { machine.machine_type() })
    }

    fn make_model(&self) -> MachineMakeModel {
        for_all!(|self, machine| { machine.make_model() })
    }

    fn max_part_volume(&self) -> Option<Volume> {
        for_all!(|self, machine| { machine.max_part_volume() })
    }
}

impl crate::Control for AnyMachine {
    type Error = anyhow::Error;
    type MachineInfo = AnyMachineInfo;

    async fn machine_info(&self) -> Result<AnyMachineInfo> {
        for_all!(|self, machine| { Ok(machine.machine_info().await?.into()) })
    }

    async fn emergency_stop(&self) -> Result<()> {
        for_all!(|self, machine| { machine.emergency_stop().await })
    }

    async fn stop(&self) -> Result<()> {
        for_all!(|self, machine| { machine.stop().await })
    }
}
