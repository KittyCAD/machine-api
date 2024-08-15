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

    /// No-op Machine
    Noop(crate::noop::Noop),
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

    /// No-op Machine Info
    Noop(crate::noop::MachineInfo),
}

macro_rules! def_machine_stubs {
    (if $feature:expr, $name:ident($machine:path, $machine_info:path)) => {
        #[cfg(feature = $feature)]
        impl From<$machine> for AnyMachine {
            fn from(machine: $machine) -> Self {
                Self::$name(machine)
            }
        }

        #[cfg(feature = $feature)]
        impl From<$machine_info> for AnyMachineInfo {
            fn from(machine: $machine_info) -> Self {
                Self::$name(machine)
            }
        }
    };
    ($name:ident($machine:path, $machine_info:path)) => {
        impl From<$machine> for AnyMachine {
            fn from(machine: $machine) -> Self {
                Self::$name(machine)
            }
        }

        impl From<$machine_info> for AnyMachineInfo {
            fn from(machine: $machine_info) -> Self {
                Self::$name(machine)
            }
        }
    };
}

def_machine_stubs!(if "bambu",     BambuX1Carbon(crate::bambu::X1Carbon, crate::bambu::PrinterInfo));
def_machine_stubs!(if "moonraker", Moonraker(crate::moonraker::Client, crate::moonraker::MachineInfo));
def_machine_stubs!(if "serial",    Usb(crate::gcode::Usb, crate::gcode::UsbMachineInfo));

def_machine_stubs!(Noop(crate::noop::Noop, crate::noop::MachineInfo));

macro_rules! for_all {
    (|$slf:ident, $machine:ident| $body:block) => {
        match $slf {
            #[cfg(feature = "bambu")]
            Self::BambuX1Carbon($machine) => $body,

            #[cfg(feature = "moonraker")]
            Self::Moonraker($machine) => $body,

            #[cfg(feature = "serial")]
            Self::Usb($machine) => $body,

            Self::Noop($machine) => $body,
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

    async fn emergency_stop(&mut self) -> Result<()> {
        for_all!(|self, machine| { machine.emergency_stop().await })
    }

    async fn stop(&mut self) -> Result<()> {
        for_all!(|self, machine| { machine.stop().await })
    }
}
