use super::{Config, MachineConfig};
use machine_api::{usb, Discover, Machine};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

impl Config {
    pub async fn spawn_discover_usb(&self, machines: Arc<RwLock<HashMap<String, RwLock<Machine>>>>) {
        let discovery = usb::UsbDiscovery::new(
            self.machines
                .iter()
                .map(|(key, config)| {
                    if let MachineConfig::Usb(config) = config {
                        Some((key.clone(), config.clone()))
                    } else {
                        None
                    }
                })
                .flatten()
                .collect::<HashMap<_, _>>(),
        );

        tokio::spawn(async move {
            let _ = discovery.discover(machines).await;
            panic!("usb discovery broke");
        });
    }
}
