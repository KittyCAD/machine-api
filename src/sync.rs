use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{Control, HardwareConfiguration, MachineState};

/// Wrapper around an `Arc<Mutex<Control>>`, which helpfully will handle
/// the locking to expose a [Control] without the caller having to care
/// that this is a shared handle.
pub struct SharedMachine<ControlT>(
    /// Underlying shared Control channel.
    pub Arc<Mutex<ControlT>>,
)
where
    ControlT: Control;

impl<ControlT> From<Arc<Mutex<ControlT>>> for SharedMachine<ControlT>
where
    ControlT: Control,
{
    fn from(inner: Arc<Mutex<ControlT>>) -> Self {
        Self(inner)
    }
}

impl<ControlT> Control for SharedMachine<ControlT>
where
    ControlT: Control,
{
    type Error = ControlT::Error;
    type MachineInfo = ControlT::MachineInfo;

    async fn machine_info(&self) -> Result<Self::MachineInfo, Self::Error> {
        self.0.lock().await.machine_info().await
    }
    async fn emergency_stop(&mut self) -> Result<(), Self::Error> {
        self.0.lock().await.emergency_stop().await
    }
    async fn stop(&mut self) -> Result<(), Self::Error> {
        self.0.lock().await.stop().await
    }
    async fn healthy(&self) -> bool {
        self.0.lock().await.healthy().await
    }
    async fn state(&self) -> Result<MachineState, Self::Error> {
        self.0.lock().await.state().await
    }
    async fn progress(&self) -> Result<Option<f64>, Self::Error> {
        self.0.lock().await.progress().await
    }
    async fn hardware_configuration(&self) -> Result<HardwareConfiguration, Self::Error> {
        self.0.lock().await.hardware_configuration().await
    }
}
