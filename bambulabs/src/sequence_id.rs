//! The sequence id type.

use std::sync::atomic::AtomicU32;

use anyhow::Result;
use parse_display::{Display, FromStr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    /// The atomic counter for sequence IDs.
    pub static ref ATOMIC_COUNTER: AtomicU32 = AtomicU32::new(0);
}

/// The sequence id type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema, Display, FromStr)]
#[serde(untagged)]
#[display("{0}")]
pub enum SequenceId {
    /// A string sequence id.
    String(String),
    /// An integer sequence id.
    Integer(u32),
}

impl SequenceId {
    /// Create a new sequence id.
    pub fn new() -> Self {
        if cfg!(test) {
            Self::Integer(1)
        } else {
            Self::Integer(ATOMIC_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
        }
    }

    /// Get the sequence id as a u32.
    pub fn as_u32(&self) -> Result<u32> {
        match self {
            Self::String(s) => s.parse().map_err(Into::into),
            Self::Integer(i) => Ok(*i),
        }
    }
}

impl Default for SequenceId {
    fn default() -> Self {
        Self::new()
    }
}
