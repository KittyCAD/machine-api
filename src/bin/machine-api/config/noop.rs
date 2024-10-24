use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use machine_api::{noop, slicer, Machine, MachineMakeModel, MachineType, Volume};
use tokio::sync::RwLock;

use super::{Config, MachineConfig};

impl Config {
    pub async fn create_noop(
        &self,
        channel: tokio::sync::mpsc::Sender<String>,
        machines: Arc<RwLock<HashMap<String, RwLock<Machine>>>>,
    ) -> Result<()> {
        for (key, config) in self
            .machines
            .iter()
            .filter_map(|(key, config)| {
                if let MachineConfig::Noop(config) = config {
                    Some((key.clone(), config.clone()))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>()
        {
            machines.write().await.insert(
                key.clone(),
                RwLock::new(Machine::new(
                    noop::Noop::new(
                        config.clone(),
                        MachineMakeModel {
                            manufacturer: Some("Zoo Corporation".to_owned()),
                            model: Some("Null Machine".to_owned()),
                            serial: Some("Cheerios".to_owned()),
                        },
                        MachineType::FusedDeposition,
                        Some(Volume {
                            width: 500.0,
                            depth: 600.0,
                            height: 700.0,
                        }),
                    ),
                    slicer::noop::Slicer::new(),
                )),
            );
            channel.send(key.clone()).await?;
        }
        Ok(())
    }
}
