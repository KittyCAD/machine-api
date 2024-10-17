use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use machine_api::{moonraker, Machine, MachineMakeModel};
use tokio::sync::RwLock;

use super::{Config, MachineConfig};

impl Config {
    pub async fn create_moonraker(
        &self,
        channel: tokio::sync::mpsc::Sender<String>,
        machines: Arc<RwLock<HashMap<String, RwLock<Machine>>>>,
    ) -> Result<()> {
        for (key, config) in self
            .machines
            .iter()
            .filter_map(|(key, config)| {
                if let MachineConfig::Moonraker(config) = config {
                    Some((key.clone(), config.clone()))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>()
        {
            let slicer = config.slicer.load()?;
            let (manufacturer, model) = config.variant.get_manufacturer_model();

            machines.write().await.insert(
                key.clone(),
                RwLock::new(Machine::new(
                    moonraker::Client::new(
                        &config.endpoint.clone(),
                        MachineMakeModel {
                            manufacturer,
                            model,
                            serial: None,
                        },
                        config.variant.get_max_part_volume(),
                    )?,
                    slicer,
                )),
            );
            channel.send(key.clone()).await?;
        }

        Ok(())
    }
}
