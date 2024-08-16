use super::{Cli, Config};
use anyhow::Result;
use machine_api::server;
use std::collections::HashMap;

pub async fn main(_cli: &Cli, _cfg: &Config, bind: &str) -> Result<()> {
    server::serve(
        bind,
        HashMap::new(),
        // cfg.load()
        //     .await?
        //     .into_iter()
        //     .map(|(machine_id, machine)| (machine_id, RwLock::new(machine)))
        //     .collect(),
    )
    .await?;

    Ok(())
}
