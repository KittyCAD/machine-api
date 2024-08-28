use crate::Machine;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

/// Context for a given server -- this contains all the informatio required
/// to serve a Machine-API request.
pub struct Context {
    /// OpenAPI schema, served at the meta-endpoint `/`, which returns the
    /// OpenAPI JSON schema representing itself.
    pub schema: serde_json::Value,

    /// List of [Machine] objects to serve via the Machine API.
    pub machines: Arc<RwLock<HashMap<String, RwLock<Machine>>>>,
}
