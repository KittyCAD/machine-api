use super::{Cli, Config};
use anyhow::Result;
use machine_api::server;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

pub async fn main(_cli: &Cli, cfg: &Config, bind: &str) -> Result<()> {
    let machines = Arc::new(RwLock::new(HashMap::new()));
    cfg.spawn_discover_usb(machines.clone()).await;
    server::serve(bind, machines).await?;
    Ok(())
}
