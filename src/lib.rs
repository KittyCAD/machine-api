#![deny(missing_docs)]
#![deny(missing_copy_implementations)]
#![deny(trivial_casts)]
#![deny(trivial_numeric_casts)]
#![deny(unused_import_braces)]
#![deny(unused_qualifications)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]

//! This crate implements support for taking designed parts, and producing
//! real-world constructions of those parts.

mod traits;

#[cfg(feature = "bambu")]
mod bambu;

#[cfg(feature = "formlabs")]
mod formlabs;

#[cfg(feature = "moonraker")]
mod moonraker;

pub use traits::{DesignFile, GcodeMachine, Machine, MachineSlicer, SuspendMachine};
