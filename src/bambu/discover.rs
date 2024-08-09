use super::{Config, X1Carbon};
use crate::{Discover as DiscoverTrait, MachineInfo as MachineInfoTrait, MachineMakeModel, MachineType};
use anyhow::Result;
use std::{
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
};
// use bambulabs::{client::Client, command::Command};
use dashmap::DashMap;
use tokio::net::UdpSocket;

const BAMBU_X1_CARBON_URN: &str = "urn:bambulab-com:device:3dprinter:1";

/// Information regarding a discovered X1 Carbon.
#[derive(Debug, Clone)]
pub struct PrinterInfo {
    /// Make and model of the PrinterInfo. This is accessed through the
    /// `MachineMakeModel` trait.
    make_model: MachineMakeModel,

    /// The hostname of the printer.
    pub hostname: Option<String>,

    /// The IP address of the printer.
    pub ip: IpAddr,

    /// The port of the printer.
    pub port: Option<u16>,
}

/// Handle to discover connected Bambu Labs printers.
pub struct Discover {
    printers: DashMap<String, X1Carbon>,
    config: Config,
}

// TODO: in the future, control maybe should be an enum of specific control
// types, which itself implements MachineControl.

impl MachineInfoTrait for PrinterInfo {
    type Error = anyhow::Error;

    fn machine_type(&self) -> MachineType {
        MachineType::Stereolithography
    }

    fn make_model(&self) -> MachineMakeModel {
        self.make_model.clone()
    }
}

impl DiscoverTrait for Discover {
    type Error = anyhow::Error;
    type MachineInfo = PrinterInfo;

    async fn discovered(&self) -> Result<Vec<PrinterInfo>> {
        Ok(self
            .printers
            .iter()
            .map(|printer| printer.value().info.clone())
            .collect())
    }

    async fn discover(&self) -> Result<()> {
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

            if self.printers.contains_key(&ip.to_string()) {
                tracing::debug!("Printer already discovered, skipping");
                continue;
            }

            let Some(name) = name else {
                tracing::warn!("No name found for printer at {}", ip);
                continue;
            };

            let Some(config) = self.config.get_machine_config(&name.to_string()) else {
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

            let handle = X1Carbon {
                info,
                client: Arc::new(client),
            };
            self.printers.insert(ip.to_string(), handle);
        }

        Ok(())
    }
}
