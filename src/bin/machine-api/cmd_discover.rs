use super::{Cli, Config};
use anyhow::Result;
use machine_api::{slicer, usb, Control, Discover};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

// async fn print_discovered<DiscoverT>(name: String, discover: DiscoverT)
// where
//     DiscoverT: Discover,
//     DiscoverT: Clone,
//     DiscoverT: 'static,
//     <DiscoverT::Control as Control>::MachineInfo: Send,
//     <DiscoverT::Control as Control>::MachineInfo: Debug,
// {
//     let (send, mut recv) = mpsc::channel::<<DiscoverT::Control as Control>::MachineInfo>(1);
//     let name1 = name.clone();
//     tokio::spawn(async move {
//         let name = name1;
//         while let Some(found) = recv.recv().await {
//             println!("[{}] {:?}", name, found);
//         }
//     });
//
//     tracing::debug!("starting to discover {name} devices");
//     let _ = discover.discover(send).await;
// }

pub async fn main(_cli: &Cli, cfg: &Config) -> Result<()> {
    let usb_discover = usb::UsbDiscovery::new([(
        "foo".to_owned(),
        usb::Config {
            slicer: slicer::Config::Prusa {
                config: "config/prusa/neptune4.ini".to_owned(),
            },
            baud: Some(115200),
            vendor_id: Some(0x1a86),
            product_id: Some(0x7523),
            serial: None,
            variant: usb::UsbVariant::Generic,
        },
    )]);

    let machines = Arc::new(RwLock::new(HashMap::new()));
    usb_discover.discover(machines.clone()).await?;

    Ok(())
}
