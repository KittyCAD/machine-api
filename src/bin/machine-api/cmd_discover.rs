use super::{Cli, Config};
use anyhow::Result;
use machine_api::{Control, Discover};
use std::fmt::Debug;
use tokio::{sync::mpsc, task::JoinSet};

async fn print_discovered<DiscoverT>(name: String, discover: DiscoverT)
where
    DiscoverT: Discover,
    DiscoverT: Clone,
    DiscoverT: 'static,
    <DiscoverT::Control as Control>::MachineInfo: Send,
    <DiscoverT::Control as Control>::MachineInfo: Debug,
{
    let (send, mut recv) = mpsc::channel::<<DiscoverT::Control as Control>::MachineInfo>(1);
    let name1 = name.clone();
    tokio::spawn(async move {
        let name = name1;
        while let Some(found) = recv.recv().await {
            println!("[{}] {:?}", name, found);
        }
    });

    tracing::debug!("starting to discover {name} devices");
    let _ = discover.discover(send).await;
}

pub async fn main(_cli: &Cli, cfg: &Config) -> Result<()> {
    let mut join_set = JoinSet::new();

    if let Some(discover) = cfg.load_discover_usb().await? {
        join_set.spawn(async move {
            print_discovered("usb".to_owned(), discover).await;
        });
    };

    if let Some(discover) = cfg.load_discover_moonraker().await? {
        join_set.spawn(async move {
            print_discovered("moonraker".to_owned(), discover).await;
        });
    };

    join_set.join_next().await;

    Ok(())
}
