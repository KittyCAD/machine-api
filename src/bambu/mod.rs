//! This module contains support for printing to Bambu Lab 3D printers.

mod control;
mod discover;
mod temperature;

use std::{net::IpAddr, sync::Arc};

use bambulabs::client::Client;
pub use discover::{BambuDiscover, BambuVariant, Config};

use crate::MachineMakeModel;

/// Control channel handle to a Bambu Labs printer.
#[derive(Clone)]
pub struct Bambu {
    client: Arc<Client>,
    info: PrinterInfo,
}

/// Information regarding a discovered Bambu Labs printer.
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
