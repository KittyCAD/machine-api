use super::{Config, X1Carbon};
use crate::{Discover as DiscoverTrait, MachineInfo as MachineInfoTrait, MachineMakeModel, MachineType};
use anyhow::Result;
// use bambulabs::{client::Client, command::Command};
use dashmap::DashMap;
// use tokio::net::UdpSocket;

#[derive(Debug, Clone, Copy)]
pub struct PrinterInfo {}

/// Handle to discover connected Bambu Labs printers.
pub struct Discover {
    printers: DashMap<String, PrinterInfo>,
    config: Config,
}

// TODO: in the future, control maybe should be an enum of specific control
// types, which itself implements MachineControl.

impl MachineInfoTrait for PrinterInfo {
    type Error = anyhow::Error;
    type Control = X1Carbon;

    fn machine_type(&self) -> MachineType {
        MachineType::Stereolithography
    }

    fn make_model(&self) -> MachineMakeModel {
        unimplemented!();
    }

    async fn control(&self) -> Result<X1Carbon> {
        unimplemented!();
    }
}

impl DiscoverTrait for Discover {
    type Error = anyhow::Error;
    type MachineInfo = PrinterInfo;

    async fn discover(&self) -> Result<()> {
        unimplemented!();
    }

    async fn discovered(&self) -> Result<Vec<PrinterInfo>> {
        Ok(vec![])
    }
}
