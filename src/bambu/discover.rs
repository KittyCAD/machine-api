use super::{PrinterInfo, X1Carbon};
use crate::{slicer, Discover as DiscoverTrait, Machine, MachineMakeModel};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
};
use tokio::{net::UdpSocket, sync::RwLock};

/// Specific make/model of Bambu device.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum BambuVariant {
    /// Bambu Labs X1 Carbon printer
    X1Carbon,
}

/// Configuration block for a Bambu device.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    /// Slicer configuration for created Machine.
    pub slicer: slicer::Config,

    /// Information regarding the specific make/model of device.
    pub variant: BambuVariant,

    /// The printer's name
    pub name: String,

    /// The access code for the printer.
    pub access_code: String,
}

const BAMBU_X1_CARBON_URN: &str = "urn:bambulab-com:device:3dprinter:1";

/// Handle to discover connected Bambu Labs printers.
pub struct X1CarbonDiscover {
    config: HashMap<String, Config>,
}

impl X1CarbonDiscover {
    /// Return a new Discover handle using the provided Configuration
    /// struct [Config].
    pub fn new<ConfigsT: Into<HashMap<String, Config>>>(cfgs: ConfigsT) -> Self {
        X1CarbonDiscover { config: cfgs.into() }
    }

    fn config_for_name(&self, name: &str) -> Option<(String, Config)> {
        self.config
            .iter()
            .find(|(_, config)| config.name == name)
            .map(|(k, v)| (k.clone(), v.clone()))
    }
}

impl DiscoverTrait for X1CarbonDiscover {
    type Error = anyhow::Error;

    async fn discover(&self, printers: Arc<RwLock<HashMap<String, RwLock<Machine>>>>) -> Result<()> {
        if self.config.is_empty() {
            tracing::debug!("no bambu devices configured, shutting down bambu scans");
            return Ok(());
        }

        tracing::info!("Spawning Bambu discovery task");

        // Any interface, port 2021, which is a non-standard port for any kind of UPnP/SSDP protocol.
        // Incredible.
        let any = (Ipv4Addr::new(0, 0, 0, 0), 2021);
        let socket = UdpSocket::bind(any).await?;

        let mut socket_buf = [0u8; 1536];

        while let Ok(n) = socket.recv(&mut socket_buf).await {
            // The SSDP/UPnP frames we're looking for from Bambu printers are pure ASCII, so we don't
            // mind if we end up with garbage in the resulting string. Note that other SSDP packets from
            // e.g. macOS Bonjour(?) do contain binary data which means this conversion isn't suitable
            // for them.
            let udp_payload = String::from_utf8_lossy(&socket_buf[0..n]);

            // Iterate through all non-blank lines in the payload
            let mut lines = udp_payload.lines().filter_map(|l| {
                let l = l.trim();

                if l.is_empty() {
                    None
                } else {
                    Some(l)
                }
            });

            // First line is a different format to the rest. We also need to check this for the message
            // type the Bambu printer emits, which is "NOTIFY * HTTP/1.1"
            let Some(header) = lines.next() else {
                tracing::debug!("Bad UPnP");

                continue;
            };

            // We don't need to parse this properly :)))))
            if header != "NOTIFY * HTTP/1.1" {
                tracing::trace!("Not a notify, ignoring header {:?}", header);

                continue;
            }

            let mut urn = None;
            let mut name = None;
            let mut ip: Option<IpAddr> = None;
            let mut serial = None;
            // TODO: This is probably the secure MQTT port 8883 but we need to test that assumption
            #[allow(unused_mut)]
            let mut port = None;

            for line in lines {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                let Some((token, rest)) = line.split_once(':') else {
                    tracing::debug!("Bad token line {}", line);

                    continue;
                };

                let token = token.trim();
                let rest = rest.trim();

                tracing::trace!("----> Token {}: {}", token, rest);

                match token {
                    "Location" => ip = Some(rest.parse().expect("Bad IP")),
                    "DevName.bambu.com" => name = Some(rest.to_owned()),
                    "USN" => serial = Some(rest.to_owned()),
                    "NT" => urn = Some(rest.to_owned()),
                    // Ignore everything else
                    _ => (),
                }
            }

            let Some(ip) = ip else {
                tracing::warn!("No IP address present for printer name {:?} (URN {:?})", name, urn);

                continue;
            };

            // A little extra validation: check the URN is a Bambu printer. This is currently only
            // tested against the Bambu Lab X1 Carbon with AMS.
            if urn != Some(BAMBU_X1_CARBON_URN.to_string()) {
                tracing::warn!(
                    "Printer doesn't appear to be an X1 Carbon: URN {:?} does not match {}",
                    urn,
                    BAMBU_X1_CARBON_URN
                );

                continue;
            }

            if printers.read().await.contains_key(&ip.to_string()) {
                tracing::debug!("Printer already discovered, skipping");
                continue;
            }

            let Some(name) = name else {
                tracing::warn!("No name found for printer at {}", ip);
                continue;
            };

            let Some((machine_api_id, config)) = self.config_for_name(&name) else {
                tracing::warn!("No config found for printer at {}", ip);
                continue;
            };

            // Add a mqtt client for this printer.
            let serial = serial.as_deref().unwrap_or_default();

            let client =
                bambulabs::client::Client::new(ip.to_string(), config.access_code.to_string(), serial.to_string())?;
            let mut cloned_client = client.clone();
            tokio::spawn(async move {
                cloned_client.run().await.unwrap();
            });

            // At this point, we have a valid (as long as the parsing above is strict enough lmao)
            // collection of data that represents a Bambu X1 Carbon.
            let info = PrinterInfo {
                hostname: Some(name),
                ip,
                port,
                make_model: MachineMakeModel {
                    manufacturer: Some("Bambu Lab".to_owned()),
                    // We can hard code this for now as we check the URN above (and assume the URN is
                    // unique to the X1 carbon)
                    model: Some("X1 Carbon".to_owned()),
                    serial: Some(serial.to_string()),
                },
            };

            let Ok(slicer) = config.slicer.load() else {
                tracing::error!("slicer failed to load");
                continue;
            };

            printers.write().await.insert(
                machine_api_id,
                RwLock::new(Machine::new(
                    X1Carbon {
                        info,
                        client: Arc::new(client),
                    },
                    slicer,
                )),
            );
        }

        Ok(())
    }
}
