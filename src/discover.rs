use crate::{Control, MachineInfo};
use anyhow::Result;
use std::{collections::HashMap, future::Future, hash::Hash, sync::Arc};
use tokio::sync::{mpsc::Sender, Mutex};

/// Trait implemented by schemes that can dynamically resolve Machines that can
/// be controlled by the `machine-api`.
pub trait Discover {
    /// Error type returned by this trait, and any relient traits.
    type Error;

    /// Underlying type containing information about the discovered printer.
    type MachineInfo: MachineInfo;

    /// Underlying type allowing for control of a printer.
    type Control: Control;

    /// Discover all rachable printers which are made discoverable through
    /// some mechanism (mDNS, USB, etc).
    ///
    /// This will continuously search for printers until the program is
    /// stopped. You likely want to spawn this on a separate tokio task.
    ///
    /// When a new Machine is found, the callback will be invoked.
    fn discover(&self, found: Sender<Self::MachineInfo>) -> impl Future<Output = Result<(), Self::Error>>;

    /// Connect to a discovered printer.
    fn connect(
        &self,
        machine: Self::MachineInfo,
    ) -> impl Future<Output = Result<Arc<Mutex<Self::Control>>, Self::Error>>;
}

/// SimpleDiscovery is a [Discover] traited wrapper for internal use
/// in implementnig Discover.
#[derive(Debug)]
pub(crate) struct SimpleDiscovery<KeyT, ConfigT, ControlT>
where
    KeyT: Eq,
    KeyT: Hash,
    KeyT: Clone,
    ConfigT: Clone,
    ControlT: Control,
    ControlT: Send,
    ControlT::MachineInfo: Clone,
    ControlT::MachineInfo: Send,
    ControlT::MachineInfo: PartialEq,
{
    known_machines: HashMap<KeyT, ConfigT>,
    found_machine_infos: Arc<Mutex<HashMap<KeyT, ControlT::MachineInfo>>>,
    found_machines: Arc<Mutex<HashMap<KeyT, Arc<Mutex<ControlT>>>>>,
    sender: Sender<ControlT::MachineInfo>,
}

impl<KeyT, ConfigT, ControlT> Clone for SimpleDiscovery<KeyT, ConfigT, ControlT>
where
    KeyT: Eq,
    KeyT: Hash,
    KeyT: Clone,
    ConfigT: Clone,
    ControlT: Control,
    ControlT: Send,
    ControlT::MachineInfo: Clone,
    ControlT::MachineInfo: Send,
    ControlT::MachineInfo: PartialEq,
{
    fn clone(&self) -> Self {
        Self {
            known_machines: self.known_machines.clone(),
            found_machine_infos: self.found_machine_infos.clone(),
            found_machines: self.found_machines.clone(),
            sender: self.sender.clone(),
        }
    }
}

impl<KeyT, ConfigT, ControlT> SimpleDiscovery<KeyT, ConfigT, ControlT>
where
    KeyT: Eq,
    KeyT: Hash,
    KeyT: Clone,
    ConfigT: Clone,
    ControlT: Control,
    ControlT: Send,
    ControlT::MachineInfo: Clone,
    ControlT::MachineInfo: Send,
    ControlT::MachineInfo: PartialEq,
{
    /// Create a new SimpleDiscovery with the preconfigured machines.
    pub fn new(known_machines: HashMap<KeyT, ConfigT>, sender: Sender<ControlT::MachineInfo>) -> Self {
        Self {
            known_machines,
            found_machine_infos: Arc::new(Mutex::new(HashMap::new())),
            found_machines: Arc::new(Mutex::new(HashMap::new())),
            sender,
        }
    }

    /// get the machine by key.
    pub async fn machine(&self, key: &KeyT) -> Option<Arc<Mutex<ControlT>>> {
        self.found_machines.lock().await.get(key).cloned()
    }

    /// return true if the key has been found already.
    pub async fn machine_info(&self, key: &KeyT) -> Option<ControlT::MachineInfo> {
        self.found_machine_infos.lock().await.get(key).cloned()
    }

    /// return true if the key has been found already.
    pub async fn machine_config(&self, key: &KeyT) -> Option<ConfigT> {
        self.known_machines.get(key).cloned()
    }

    pub async fn insert(&self, key: KeyT, mi: ControlT::MachineInfo, control: ControlT) {
        tracing::trace!("locking found_machine_infos");
        self.found_machine_infos.lock().await.insert(key.clone(), mi.clone());
        tracing::trace!("locking found_machines");
        self.found_machines
            .lock()
            .await
            .insert(key.clone(), Arc::new(Mutex::new(control)));
        tracing::debug!("inserted new machine");
        let _ = self.sender.send(mi).await;
        tracing::trace!("info broadcasted to listeners");
    }
}
