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
    pub fn new(write: WriteT) -> Result<Self> {
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
