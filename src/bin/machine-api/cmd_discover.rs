use super::{Cli, Config};
use anyhow::Result;
use machine_api::Discover;
use std::fmt::Debug;
use tokio::sync::mpsc;

async fn print_discovered<DiscoverT>(name: String, discover: DiscoverT)
where
    DiscoverT: Discover,
    DiscoverT: Clone,
    DiscoverT: 'static,
    DiscoverT::MachineInfo: Send,
    DiscoverT::MachineInfo: Debug,
{
    let (send, mut recv) = mpsc::channel::<DiscoverT::MachineInfo>(1);
    tokio::spawn(async move {
        while let Some(found) = recv.recv().await {
            println!("[{}] {:?}", name, found);
        }
    });
    let _ = discover.discover(send).await;
}

pub async fn main(_cli: &Cli, cfg: &Config) -> Result<()> {
    if let Some(discover) = cfg.load_discover_usb().await? {
        tracing::info!("loading");
        print_discovered("usb".to_owned(), discover).await;
    };

    Ok(())
}
