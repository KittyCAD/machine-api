//! This module contains support for printing to moonraker 3D printers.

use anyhow::Result;
use std::{
    pin::Pin,
    task::{Context as TaskContext, Poll},
};
use tokio::io::{AsyncWrite, AsyncWriteExt};

/// Create a handle to some [tokio::io::AsyncWrite]
pub struct Client<WriteT>
where
    WriteT: AsyncWrite,
{
    write: WriteT,
}

impl<WriteT> Client<WriteT>
where
    WriteT: AsyncWrite,
    WriteT: Unpin,
{
    /// Create a new [Client] using some underlying [tokio::io::AsyncWrite].
    pub async fn new(write: WriteT) -> Result<Self> {
        Ok(Self { write })
    }

    /// Issue a G0 stop command.
    pub async fn stop(&mut self) -> Result<()> {
        self.write_all(b"G01\n").await?;
        Ok(())
    }

    /// Issue a G112 full shutdown
    pub async fn emergency_stop(&mut self) -> Result<()> {
        self.write_all(b"G112\n").await?;
        Ok(())
    }
}

impl<WriteT> AsyncWrite for Client<WriteT>
where
    WriteT: AsyncWrite,
    WriteT: Unpin,
{
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>, buf: &[u8]) -> Poll<tokio::io::Result<usize>> {
        Pin::new(&mut self.write).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut TaskContext) -> Poll<tokio::io::Result<()>> {
        Pin::new(&mut self.write).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut TaskContext) -> Poll<tokio::io::Result<()>> {
        Pin::new(&mut self.write).poll_shutdown(cx)
    }
}

#[allow(unused_macros)]
macro_rules! gcode_machine {
    ($name:ident) => {
        /// gcode based Control interface to a Machine.
        pub struct $name<WriteT>
        where
            WriteT: AsyncWrite,
            WriteT: Unpin,
        {
            client: std::sync::Arc<tokio::sync::Mutex<$crate::gcode::Client<WriteT>>>,
        }

        impl<WriteT> $name<WriteT>
        where
            WriteT: AsyncWrite,
            WriteT: Unpin,
        {
            /// Create a new wrapper around a Client, with some extra bits.
            pub(crate) fn from_client(client: $crate::gcode::Client<WriteT>) -> Self {
                Self {
                    client: std::sync::Arc::new(tokio::sync::Mutex::new(client)),
                }
            }

            /// Return the inner client to directly interface with the
            /// machine.
            pub async fn inner(&mut self) -> tokio::sync::MutexGuard<'_, $crate::gcode::Client<WriteT>> {
                self.client.lock().await
            }
        }
    };
}
#[allow(unused_imports)]
pub(crate) use gcode_machine;

#[cfg(feature = "serial")]
mod usb {
    use super::*;
    use crate::{Control as ControlTrait, MachineInfo as MachineInfoTrait, MachineMakeModel, MachineType, Volume};
    use tokio_serial::{SerialPortBuilder, SerialPortBuilderExt, SerialStream};

    gcode_machine!(Usb);

    /// USB information
    #[derive(Debug, Clone, Copy)]
    pub struct UsbMachineInfo {}

    impl MachineInfoTrait for UsbMachineInfo {
        type Error = anyhow::Error;

        fn machine_type(&self) -> MachineType {
            MachineType::FusedDeposition
        }

        fn make_model(&self) -> MachineMakeModel {
            MachineMakeModel {
                manufacturer: None,
                model: None,
                serial: None,
            }
        }

        fn max_part_volume(&self) -> Result<Volume> {
            unimplemented!()
        }
    }

    impl<WriteT> ControlTrait for Usb<WriteT>
    where
        WriteT: AsyncWrite,
        WriteT: Unpin,
    {
        type Error = anyhow::Error;
        type MachineInfo = UsbMachineInfo;

        async fn machine_info(&self) -> Result<UsbMachineInfo> {
            unimplemented!();
        }

        async fn emergency_stop(&self) -> Result<()> {
            self.client.lock().await.emergency_stop().await
        }

        async fn stop(&self) -> Result<()> {
            self.client.lock().await.stop().await
        }
    }

    impl Usb<SerialStream> {
        /// Open a serial port.
        pub async fn open(builder: SerialPortBuilder) -> Result<Self> {
            let client = builder.open_native_async()?;
            Ok(Usb::from_client(Client::new(client).await?))
        }
    }
}

#[cfg(feature = "serial")]
pub use usb::{Usb, UsbMachineInfo};
