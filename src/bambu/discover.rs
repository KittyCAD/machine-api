use super::{Config, PrinterInfo, X1Carbon};
use crate::{MachineInfo as MachineInfoTrait, MachineMakeModel, MachineType, Volume};
use anyhow::Result;
use dashmap::DashMap;
use std::{
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
};
use tokio::net::UdpSocket;

const BAMBU_X1_CARBON_URN: &str = "urn:bambulab-com:device:3dprinter:1";

/// Handle to discover connected Bambu Labs printers.
pub struct Discover {
    printers: DashMap<String, X1Carbon>,
    config: Config,
}

// TODO: in the future, control maybe should be an enum of specific control
// types, which itself implements MachineControl.

impl MachineInfoTrait for PrinterInfo {
    fn machine_type(&self) -> MachineType {
        MachineType::Stereolithography
    }

    fn make_model(&self) -> MachineMakeModel {
        self.make_model.clone()
    }

    fn max_part_volume(&self) -> Option<Volume> {
        Some(Volume {
            width: 256.0,
            height: 256.0,
            depth: 256.0,
        })
    }
}

impl Discover {
    /// Return a new Discover handle using the provided Configuration
    /// struct [Config].
    pub fn new(config: &Config) -> Self {
        Discover {
            printers: DashMap::new(),
            config: config.clone(),
        }
    }

    /// Attempt to connect to a discovered printer
    pub async fn connect(&self, machine: PrinterInfo) -> Result<X1Carbon> {
        Ok(self.printers.get(&machine.ip.to_string()).unwrap().clone())
    }

    /// Return all discovered printers
    pub async fn discovered(&self) -> Result<Vec<PrinterInfo>> {
        Ok(self
            .printers
            .iter()
            .map(|printer| printer.value().info.clone())
            .collect())
    }

    /// Run (in a tokio task!) a forever loop to discover any connected
    /// Bambu printers on the LAN.
    pub async fn discover(&self) -> Result<()> {
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