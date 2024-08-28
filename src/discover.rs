use crate::Machine;
use std::{collections::HashMap, future::Future, sync::Arc};
use tokio::sync::RwLock;

/// Discover trait implemented by backends in order to add or remove
/// configured machines.
pub trait Discover {
    /// Error type returned by the backend.
    type Error;

    /// Manage configured devices in the shared HashMap -- this will, on
    /// the called thread, scan for any known devices matching any configured
    /// devices, and add them as required. This is also responsible for
    /// cleaning up and reconnecting any handles that have gone stale.
    fn discover(
        &self,
        found: Arc<RwLock<HashMap<String, RwLock<Machine>>>>,
    ) -> impl Future<Output = Result<(), Self::Error>>;
}
