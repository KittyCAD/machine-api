use super::{Config, MachineConfig};
use anyhow::Result;
use machine_api::{bambu, Discover, Machine};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

impl Config {
    pub async fn spawn_discover_bambu(&self, machines: Arc<RwLock<HashMap<String, RwLock<Machine>>>>) -> Result<()> {
        let discovery = bambu::X1CarbonDiscover::new(
            self.machines
                .iter()
                .filter_map(|(key, config)| {
                    if let MachineConfig::Bambu(config) = config {
                        Some((key.clone(), config.clone()))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<_, _>>(),
        );

        tokio::spawn(async move {
            let _ = discovery.discover(machines).await;
        });

        Ok(())
    }
}
