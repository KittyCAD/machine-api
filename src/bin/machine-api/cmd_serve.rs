use super::{Cli, Config};
use anyhow::Result;
use machine_api::server;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

pub async fn main(_cli: &Cli, cfg: &Config, bind: &str) -> Result<()> {
    let machines = Arc::new(RwLock::new(HashMap::new()));

    cfg.spawn_discover_usb(machines.clone()).await?;
    cfg.spawn_discover_bambu(machines.clone()).await?;
    cfg.create_noop(machines.clone()).await?;
    cfg.create_moonraker(machines.clone()).await?;

    let bind_addr: SocketAddr = bind.parse()?;
    tokio::spawn(async move {
        let bind_addr = bind_addr;
        let responder = libmdns::Responder::new().unwrap();
        let _svc = responder.register(
            "_machine-api._tcp".to_owned(),
            "Machine Api Server".to_owned(),
            bind_addr.port(),
            &["path=/"],
        );

        tracing::info!(
            bind_addr = bind_addr.to_string(),
            "starting mDNS advertisement for _machine-api._tcp"
        );
    });

    server::serve(bind, machines).await?;
    Ok(())
}
