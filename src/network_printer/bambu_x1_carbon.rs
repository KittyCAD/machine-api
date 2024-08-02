use std::{
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
};

use anyhow::Result;
use bambulabs::command::Command;
use dashmap::DashMap;
use tokio::net::UdpSocket;

use crate::{
    config::BambuLabsConfig,
    network_printer::{
        Message, NetworkPrinter, NetworkPrinterHandle, NetworkPrinterInfo, NetworkPrinterManufacturer, NetworkPrinters,
    },
};

const BAMBU_X1_CARBON_URN: &str = "urn:bambulab-com:device:3dprinter:1";

pub struct BambuX1Carbon {
    pub printers: DashMap<String, NetworkPrinterHandle>,
    pub config: BambuLabsConfig,
}

impl BambuX1Carbon {
    pub fn new(config: &BambuLabsConfig) -> Self {
        Self {
            printers: DashMap::new(),
            config: config.clone(),
        }
    }
}

#[async_trait::async_trait]
impl NetworkPrinters for BambuX1Carbon {
    async fn discover(&self) -> anyhow::Result<()> {
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
                tracing::debug!("No IP address present for printer name {:?} (URN {:?})", name, urn);

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

            // Get the access code from the config.
            let Some(access_code) = self.config.get_access_code(&name.to_string()) else {
                tracing::warn!("No access code found for printer at {}", ip);
                continue;
            };

            // Add a mqtt client for this printer.
            let serial = serial.as_deref().unwrap_or_default();

            let client = bambulabs::client::Client::new(ip.to_string(), access_code.to_string(), serial.to_string())?;
            let mut cloned_client = client.clone();
            tokio::spawn(async move {
                cloned_client.run().await.unwrap();
            });

            // At this point, we have a valid (as long as the parsing above is strict enough lmao)
            // collection of data that represents a Bambu X1 Carbon.
            let info = NetworkPrinterInfo {
                hostname: Some(name),
                ip,
                port,
                manufacturer: NetworkPrinterManufacturer::Bambu,
                // We can hard code this for now as we check the URN above (and assume the URN is
                // unique to the X1 carbon)
                model: Some(String::from("Bambu Lab X1 Carbon")),
                serial: Some(serial.to_string()),
            };

            let handle = NetworkPrinterHandle {
                info,
                client: Arc::new(Box::new(BambuX1CarbonPrinter {
                    client: Arc::new(client),
                })),
            };
            self.printers.insert(ip.to_string(), handle);
        }

        Ok(())
    }

    fn list(&self) -> anyhow::Result<Vec<NetworkPrinterInfo>> {
        Ok(self
            .printers
            .iter()
            .map(|printer| printer.value().info.clone())
            .collect())
    }

    fn list_handles(&self) -> Result<Vec<NetworkPrinterHandle>> {
        Ok(self.printers.iter().map(|printer| printer.value().clone()).collect())
    }
}

pub struct BambuX1CarbonPrinter {
    pub client: Arc<bambulabs::client::Client>,
}

#[async_trait::async_trait]
impl NetworkPrinter for BambuX1CarbonPrinter {
    /// Get the status of a printer.
    async fn status(&self) -> Result<Message> {
        // Get the status of the printer.
        let status = self.client.publish(Command::get_version()).await?;

        Ok(status.into())
    }

    /// Pause the current print.
    async fn pause(&self) -> Result<Message> {
        // Pause the printer.
        let pause = self.client.publish(Command::pause()).await?;

        Ok(pause.into())
    }

    /// Resume the current print.
    async fn resume(&self) -> Result<Message> {
        // Resume the printer.
        let resume = self.client.publish(Command::resume()).await?;

        Ok(resume.into())
    }

    /// Stop the current print.
    async fn stop(&self) -> Result<Message> {
        // Stop the printer.
        let stop = self.client.publish(Command::stop()).await?;

        Ok(stop.into())
    }

    /// Set the led on or off.
    async fn set_led(&self, on: bool) -> Result<Message> {
        let light = self.client.publish(Command::set_chamber_light(on.into())).await?;

        Ok(light.into())
    }

    /// Get the accessories.
    async fn accessories(&self) -> Result<Message> {
        // Get the accessories of the printer.
        let accessories = self.client.publish(Command::get_accessories()).await?;

        Ok(accessories.into())
    }

    /// Print a file.
    async fn print(&self, _file: &str) -> Result<()> {
        unimplemented!()
    }

    /// Upload a file.
    async fn upload_file(&self, file: &std::path::Path) -> Result<()> {
        Ok(self.client.upload_file(file).await?)
    }
}
