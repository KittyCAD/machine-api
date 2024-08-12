use crate::{MachineInfo, MachineMakeModel, MachineType, Volume};
use anyhow::Result;

/// Any is any supported machine.
pub enum Any {
    /// Bambu Labs X1 Carbon
    #[cfg(feature = "bambu")]
    BambuX1Carbon(crate::bambu::X1Carbon),

    /// Generic Moonraker type printer
    #[cfg(feature = "moonraker")]
    Moonraker(crate::moonraker::Client),
}

pub enum AnyMachineInfo {
    /// Bambu Labs X1 Carbon
    #[cfg(feature = "bambu")]
    BambuX1Carbon(crate::bambu::PrinterInfo),

    /// Generic Moonraker type printer
    #[cfg(feature = "moonraker")]
    Moonraker(crate::moonraker::MachineInfo),
}

#[cfg(feature = "bambu")]
mod _bambu {
    use super::*;

    impl From<crate::bambu::X1Carbon> for Any {
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

    impl From<crate::moonraker::Client> for Any {
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

macro_rules! for_all {
    (|$slf:ident, $machine:ident| $body:block) => {
        match $slf {
            #[cfg(feature = "bambu")]
            Self::BambuX1Carbon($machine) => $body,

            #[cfg(feature = "moonraker")]
            Self::Moonraker($machine) => $body,
        }
    };
}

impl MachineInfo for AnyMachineInfo {
    type Error = anyhow::Error;

    fn machine_type(&self) -> MachineType {
        for_all!(|self, machine| { machine.machine_type() })
    }

    fn make_model(&self) -> MachineMakeModel {
        for_all!(|self, machine| { machine.make_model() })
    }

    fn max_part_volume(&self) -> Result<Volume> {
        for_all!(|self, machine| { machine.max_part_volume() })
    }
}

impl crate::Control for Any {
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
