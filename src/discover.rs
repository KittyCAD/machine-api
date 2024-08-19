// use crate::{AnyMachine, AnyMachineInfo, Control};
// use anyhow::Result;
// use std::{collections::HashMap, future::Future, hash::Hash, sync::Arc};
// use tokio::sync::{mpsc::Sender, Mutex};

/// Trait implemented by schemes that can dynamically resolve Machines that can
/// be controlled by the `machine-api`.
///
/// Discovery happens through a few set phases.
///
/// 1. A tokio task is spawned (`discover`), which is used to scan for
///    devices as they appear.
///
/// 2. Each time a device is found, `DiscoveryInformation` will be sent
///    back, even if it's already configured and being used, or if
///    the device has no matching configuration -- and it'll be reported.
///
/// 3. Each time DiscoveryInformation is reported, it will be matched against
///    all known config(s). If it matches a config, it'll be constructed into
///    an impl Control, which will be reported.
///
pub trait Discover {
    /// Error type returned by the implementer.
    type Error;

    /// Information about the discovered device -- this will be returned
    /// even if the discovered node has no matching configuration block.
    type DiscoveryInformation;
}

// /// SimpleDiscovery is a [Discover] traited wrapper for internal use
// /// in implementnig Discover.
// #[derive(Debug)]
// pub(crate) struct SimpleDiscovery<KeyT, ConfigT, ControlT>
// where
//     KeyT: Eq,
//     KeyT: Hash,
//     KeyT: Clone,
//     ConfigT: Clone,
//     ControlT: Control,
//     ControlT: Send,
//     ControlT::MachineInfo: Clone,
//     ControlT::MachineInfo: Send,
//     ControlT::MachineInfo: PartialEq,
// {
//     known_machines: HashMap<KeyT, ConfigT>,
//     found_machine_infos: Arc<Mutex<HashMap<KeyT, ControlT::MachineInfo>>>,
//     found_machines: Arc<Mutex<HashMap<KeyT, Arc<Mutex<ControlT>>>>>,
//     sender: Sender<ControlT::MachineInfo>,
// }
//
// impl<KeyT, ConfigT, ControlT> Clone for SimpleDiscovery<KeyT, ConfigT, ControlT>
// where
//     KeyT: Eq,
//     KeyT: Hash,
//     KeyT: Clone,
//     ConfigT: Clone,
//     ControlT: Control,
//     ControlT: Send,
//     ControlT::MachineInfo: Clone,
//     ControlT::MachineInfo: Send,
//     ControlT::MachineInfo: PartialEq,
// {
//     fn clone(&self) -> Self {
//         Self {
//             known_machines: self.known_machines.clone(),
//             found_machine_infos: self.found_machine_infos.clone(),
//             found_machines: self.found_machines.clone(),
//             sender: self.sender.clone(),
//         }
//     }
// }
//
// impl<KeyT, ConfigT, ControlT> SimpleDiscovery<KeyT, ConfigT, ControlT>
// where
//     KeyT: Eq,
//     KeyT: Hash,
//     KeyT: Clone,
//     ConfigT: Clone,
//     ControlT: Control,
//     ControlT: Send,
//     ControlT::MachineInfo: Clone,
//     ControlT::MachineInfo: Send,
//     ControlT::MachineInfo: PartialEq,
// {
//     /// Create a new SimpleDiscovery with the preconfigured machines.
//     pub fn new(known_machines: HashMap<KeyT, ConfigT>, sender: Sender<ControlT::MachineInfo>) -> Self {
//         Self {
//             known_machines,
//             found_machine_infos: Arc::new(Mutex::new(HashMap::new())),
//             found_machines: Arc::new(Mutex::new(HashMap::new())),
//             sender,
//         }
//     }
//
//     /// get the machine by key.
//     pub async fn machine(&self, key: &KeyT) -> Option<Arc<Mutex<ControlT>>> {
//         self.found_machines.lock().await.get(key).cloned()
//     }
//
//     /// return true if the key has been found already.
//     pub async fn machine_info(&self, key: &KeyT) -> Option<ControlT::MachineInfo> {
//         self.found_machine_infos.lock().await.get(key).cloned()
//     }
//
//     /// return true if the key has been found already.
//     pub async fn machine_config(&self, key: &KeyT) -> Option<ConfigT> {
//         self.known_machines.get(key).cloned()
//     }
//
//     pub async fn insert(&self, key: KeyT, mi: ControlT::MachineInfo, control: ControlT) {
//         tracing::trace!("locking found_machine_infos");
//         self.found_machine_infos.lock().await.insert(key.clone(), mi.clone());
//         tracing::trace!("locking found_machines");
//         self.found_machines
//             .lock()
//             .await
//             .insert(key.clone(), Arc::new(Mutex::new(control)));
//         tracing::debug!("inserted new machine");
//         let _ = self.sender.send(mi).await;
//         tracing::trace!("info broadcasted to listeners");
//     }
// }
//
// /// StaticDiscover is a static list of [AnyMachine] that implements the
// /// [Discover]
// #[derive(Clone)]
// pub struct StaticDiscover(Vec<Arc<Mutex<AnyMachine>>>);
//
// impl StaticDiscover {
//     /// Create a new static discovery system
//     pub fn new(machines: Vec<Arc<Mutex<AnyMachine>>>) -> Self {
//         Self(machines)
//     }
// }
//
// impl Discover for StaticDiscover {
//     type Error = anyhow::Error;
//     type Control = AnyMachine;
//
//     async fn discover(&self, found: Sender<AnyMachineInfo>) -> Result<()> {
//         for machine in self.0.iter() {
//             // fire once for every static config
//             found.send(machine.lock().await.machine_info().await?).await?;
//         }
//
//         Ok(())
//     }
//
//     async fn connect(&self, mi: AnyMachineInfo) -> Result<Arc<Mutex<AnyMachine>>> {
//         for machine in self.0.iter() {
//             if mi == machine.lock().await.machine_info().await? {
//                 return Ok(machine.clone());
//             }
//         }
//         anyhow::bail!("machine not found");
//     }
// }
