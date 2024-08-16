//! This module contains support for printing to moonraker 3D printers.

#[cfg(feature = "serial")]
mod usb;

#[cfg(feature = "serial")]
pub use usb::{Usb, UsbDiscover, UsbHardwareMetadata, UsbMachineInfo};

use anyhow::Result;
use std::{
    pin::Pin,
    task::{Context as TaskContext, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, ReadBuf};

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
    pub fn new(write: WriteT, read: ReadT) -> Self {
        Self {
            write,
            read: BufReader::new(read),
        }
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

    /// Get the underlying ReadT to read directly on the underlying channel.
    pub fn get_read(&mut self) -> &mut BufReader<ReadT> {
        &mut self.read
    }

    /// Get the underlying WriteT to write directly on the underlying channel.
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
