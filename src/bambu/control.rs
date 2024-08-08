use crate::{Control as ControlTrait, Volume};
use anyhow::Result;
use bambulabs::client::Client;
use std::sync::Arc;

/// Control channel handle to a Bambu Labs X1 Carbon.
#[derive(Clone)]
pub struct X1Carbon {
    client: Arc<Client>,
}

impl X1Carbon {
    /// Return a borrow of the underlying Client.
    pub fn inner(&self) -> &Client {
        self.client.as_ref()
    }

    /// Get the latest status of the printer.
    pub fn get_status(&self) -> Result<Option<bambulabs::message::PushStatus>> {
        self.client.get_status()
    }

    /// Check if the printer has an AMS.
    pub fn has_ams(&self) -> Result<bool> {
        let Some(status) = self.get_status()? else {
            return Ok(false);
        };

        let Some(ams) = status.ams else {
            return Ok(false);
        };

        let Some(ams_exists) = ams.ams_exist_bits else {
            return Ok(false);
        };

        Ok(ams_exists != "0")
    }
}

impl ControlTrait for X1Carbon {
    type Error = anyhow::Error;

    async fn max_part_volume(&self) -> Result<Volume> {
        Ok(Volume {
            width: 256.0,
            height: 256.0,
            depth: 256.0,
        })
    }

    async fn emergency_stop(&self) -> Result<()> {
        unimplemented!();
    }

    async fn stop(&self) -> Result<()> {
        unimplemented!();
    }
}
