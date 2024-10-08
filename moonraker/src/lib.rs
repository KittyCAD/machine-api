#![deny(missing_docs)]
#![deny(missing_copy_implementations)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unused_import_braces)]
#![deny(unused_qualifications)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]

//! This crate implements support for interfacing with the moonraker 3d printer
//! api, proxying calls to klipper.

mod metrics;
mod print;
mod status;
mod upload;

use anyhow::Result;
pub use metrics::{ControlledTemperatureReadings, TemperatureReadings};
pub use print::InfoResponse;
pub use upload::{DeleteResponse, DeleteResponseItem, UploadResponse, UploadResponseItem};

/// Client is a moonraker instance which can accept gcode for printing.
#[derive(Clone, Debug, PartialEq)]
pub struct Client {
    pub(crate) url_base: String,
}

impl Client {
    /// Create a new Client handle to control the printer via the
    /// moonraker interface.
    pub fn new(url_base: &str) -> Result<Self> {
        tracing::debug!(base = url_base, "new");

        Ok(Self {
            url_base: url_base.to_owned(),
        })
    }
}
