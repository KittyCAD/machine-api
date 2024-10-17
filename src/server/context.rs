use std::{collections::HashMap, sync::Arc};

use prometheus_client::registry::Registry;
use tokio::sync::RwLock;

use crate::Machine;

/// Context for a given server -- this contains all the informatio required
/// to serve a Machine-API request.
pub struct Context {
    /// OpenAPI schema, served at the meta-endpoint `/`, which returns the
    /// OpenAPI JSON schema representing itself.
    pub schema: serde_json::Value,

    /// List of [Machine] objects to serve via the Machine API.
    pub machines: Arc<RwLock<HashMap<String, RwLock<Machine>>>>,

    /// Prom registry for metrics
    pub registry: Registry,
}
