use anyhow::Result;

use crate::{
    Control as ControlTrait, HardwareConfiguration, MachineInfo, MachineMakeModel, MachineState, MachineType, Volume,
};

/// AnyMachine is any supported machine.
#[non_exhaustive]
pub enum AnyMachine {
    /// Bambu Labs printer
    #[cfg(feature = "bambu")]
    Bambu(crate::bambu::Bambu),

    /// Generic Moonraker type printer
    #[cfg(feature = "moonraker")]
    Moonraker(crate::moonraker::Client),

    /// Generic USB-based gcode printer
    #[cfg(feature = "serial")]
    Usb(crate::usb::Usb),

    /// No-op Machine
    Noop(crate::noop::Noop),
}

/// AnyMachineInfo is any supported machine's MachineInfo.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum AnyMachineInfo {
    /// Bambu Labs printer
    #[cfg(feature = "bambu")]
    Bambu(crate::bambu::PrinterInfo),

    /// Generic Moonraker type printer
    #[cfg(feature = "moonraker")]
    Moonraker(crate::moonraker::MachineInfo),

    /// Generic USB-based gcode printer
    #[cfg(feature = "serial")]
    Usb(crate::usb::UsbMachineInfo),

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

def_machine_stubs!(if "bambu",     Bambu(crate::bambu::Bambu, crate::bambu::PrinterInfo));
def_machine_stubs!(if "moonraker", Moonraker(crate::moonraker::Client, crate::moonraker::MachineInfo));
def_machine_stubs!(if "serial",    Usb(crate::usb::Usb, crate::usb::UsbMachineInfo));

def_machine_stubs!(Noop(crate::noop::Noop, crate::noop::MachineInfo));

macro_rules! for_all {
    (|$slf:ident, $machine:ident| $body:block) => {
        match $slf {
            #[cfg(feature = "bambu")]
            Self::Bambu($machine) => $body,

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

impl ControlTrait for AnyMachine {
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

    async fn healthy(&self) -> bool {
        for_all!(|self, machine| { machine.healthy().await })
    }

    async fn progress(&self) -> Result<Option<f64>> {
        for_all!(|self, machine| { machine.progress().await })
    }

    async fn state(&self) -> Result<MachineState> {
        for_all!(|self, machine| { machine.state().await })
    }

    async fn hardware_configuration(&self) -> Result<HardwareConfiguration> {
        for_all!(|self, machine| { machine.hardware_configuration().await })
    }
}
