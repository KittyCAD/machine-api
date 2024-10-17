use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use machine_api::{usb, Discover, Machine};
use tokio::sync::RwLock;

use super::{Config, MachineConfig};

impl Config {
    pub async fn spawn_discover_usb(
        &self,
        channel: tokio::sync::mpsc::Sender<String>,
        machines: Arc<RwLock<HashMap<String, RwLock<Machine>>>>,
    ) -> Result<()> {
        let discovery = usb::UsbDiscovery::new(
            self.machines
                .iter()
                .filter_map(|(key, config)| {
                    if let MachineConfig::Usb(config) = config {
                        Some((key.clone(), config.clone()))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<_, _>>(),
        );

        tokio::spawn(async move {
            let _ = discovery.discover(channel, machines).await;
        });

        Ok(())
    }
}
