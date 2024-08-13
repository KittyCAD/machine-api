//! This module contains support for printing to moonraker 3D printers.

use anyhow::Result;
use std::{
    pin::Pin,
    task::{Context as TaskContext, Poll},
};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, ReadBuf};

/// Create a handle to some [tokio::io::AsyncWrite]
pub struct Client<WriteT, ReadT>
where
    WriteT: AsyncWrite,
    ReadT: AsyncRead,
{
    write: WriteT,
    read: BufReader<ReadT>,
}

impl<WriteT, ReadT> Client<WriteT, ReadT>
where
    ReadT: AsyncRead,
    ReadT: Unpin,
    WriteT: AsyncWrite,
    WriteT: Unpin,
{
    /// Create a new [Client] using some underlying [tokio::io::AsyncWrite].
    pub async fn new(write: WriteT, read: ReadT) -> Result<Self> {
        Ok(Self {
            write,
            read: BufReader::new(read),
        })
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

    ///
    pub fn get_read(&mut self) -> &mut BufReader<ReadT> {
        &mut self.read
    }

    ///
    pub fn get_write(&mut self) -> &mut WriteT {
        &mut self.write
    }
}

impl<WriteT, ReadT> AsyncWrite for Client<WriteT, ReadT>
where
    ReadT: AsyncRead,
    ReadT: Unpin,
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

// Additional trait in case the inner type is a Reader, too.

impl<WriteT, ReadT> AsyncRead for Client<WriteT, ReadT>
where
    ReadT: AsyncRead,
    ReadT: Unpin,
    WriteT: AsyncWrite,
    WriteT: Unpin,
{
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>, buf: &mut ReadBuf) -> Poll<tokio::io::Result<()>> {
        Pin::new(&mut self.read).poll_read(cx, buf)
    }
}

#[cfg(feature = "serial")]
mod usb {
    use super::*;
    use crate::{
        Control as ControlTrait, ControlGcode as ControlGcodeTrait, MachineInfo as MachineInfoTrait, MachineMakeModel,
        MachineType, TemporaryFile, Volume,
    };
    use std::sync::Arc;
    use tokio::{
        io::{ReadHalf, WriteHalf},
        sync::Mutex,
    };
    use tokio_serial::SerialStream;

    ///
    pub struct Usb {
        client: Arc<Mutex<Client<WriteHalf<SerialStream>, ReadHalf<SerialStream>>>>,

        machine_type: MachineType,
        make_model: MachineMakeModel,
        volume: Volume,
    }

    impl Usb {
        async fn wait_for_start(&self) -> Result<()> {
            loop {
                let mut line = String::new();
                if let Err(e) = self.client.lock().await.get_read().read_line(&mut line).await {
                    println!("wait_for_start err: {}", e);
                } else {
                    // Use ends with because sometimes we may still have some data left on the buffer
                    if line.trim().ends_with("start") {
                        return Ok(());
                    }
                }
            }
        }

        async fn wait_for_ok(&self) -> Result<()> {
            loop {
                let mut line = String::new();
                if let Err(e) = self.client.lock().await.get_read().read_line(&mut line).await {
                    println!("wait_for_ok err: {}", e);
                } else {
                    println!("RCVD: {}", line);
                    if line.trim() == "ok" {
                        return Ok(());
                    }
                }
            }
        }
    }

    /// Information regarding a USB connected Machine.
    pub struct UsbMachineInfo {
        machine_type: MachineType,
        make_model: MachineMakeModel,
        volume: Volume,
    }

    impl MachineInfoTrait for UsbMachineInfo {
        type Error = anyhow::Error;

        fn machine_type(&self) -> MachineType {
            self.machine_type.clone()
        }

        fn make_model(&self) -> MachineMakeModel {
            self.make_model.clone()
        }

        fn max_part_volume(&self) -> Result<Volume> {
            Ok(self.volume.clone())
        }
    }

    impl ControlTrait for Usb {
        type MachineInfo = UsbMachineInfo;
        type Error = anyhow::Error;

        async fn machine_info(&self) -> Result<UsbMachineInfo> {
            Ok(UsbMachineInfo {
                machine_type: self.machine_type.clone(),
                make_model: self.make_model.clone(),
                volume: self.volume.clone(),
            })
        }

        async fn emergency_stop(&self) -> Result<()> {
            self.client.lock().await.emergency_stop().await
        }
        async fn stop(&self) -> Result<()> {
            self.client.lock().await.stop().await
        }
    }

    impl ControlGcodeTrait for Usb {
        async fn build(&self, _job_name: &str, mut gcode: TemporaryFile) -> Result<()> {
            let mut buf = String::new();
            gcode.as_mut().read_to_string(&mut buf).await?;

            let lines: Vec<String> = buf
                .lines() // split the string into an iterator of string slices
                .map(|s| {
                    let s = String::from(s);
                    match s.split_once(';') {
                        Some((command, _)) => command.trim().to_string(),
                        None => s.trim().to_string(),
                    }
                })
                .filter(|s| !s.is_empty()) // make each slice into a string
                .collect();

            self.wait_for_start().await?;

            for line in lines.iter() {
                let msg = format!("{}\r\n", line);
                println!("writing: {}", line);
                self.client.lock().await.write_all(msg.as_bytes()).await?;
                self.wait_for_ok().await?;
            }

            Ok(())
        }
    }
}

#[cfg(feature = "serial")]
pub use usb::{Usb, UsbMachineInfo};
