//! This module contains support for printing to Bambu Lab 3D printers.

mod control;
mod discover;

pub use discover::{BambuVariant, Config, X1CarbonDiscover};

use crate::MachineMakeModel;
use bambulabs::client::Client;
use std::{net::IpAddr, sync::Arc};

/// Control channel handle to a Bambu Labs X1 Carbon.
#[derive(Clone)]
pub struct X1Carbon {
    client: Arc<Client>,
    info: PrinterInfo,
}

/// Information regarding a discovered X1 Carbon.
#[derive(Debug, Clone, PartialEq)]
pub struct PrinterInfo {
    /// Make and model of the PrinterInfo. This is accessed through the
    /// `MachineMakeModel` trait.
    make_model: MachineMakeModel,

    /// The hostname of the printer.
    pub hostname: Option<String>,

    /// The IP address of the printer.
    pub ip: IpAddr,

    /// The port of the printer.
    pub port: Option<u16>,
}
