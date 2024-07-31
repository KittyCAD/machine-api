use super::{NetworkPrinter, NetworkPrinterInfo, NetworkPrinterManufacturer};
use dashmap::DashSet;
use std::net::Ipv4Addr;
use tokio::net::UdpSocket;

const BAMBU_X1_CARBON_URN: &str = "urn:bambulab-com:device:3dprinter:1";

pub struct BambuX1Carbon {
    pub printers: DashSet<NetworkPrinterInfo>,
}

impl BambuX1Carbon {
    pub fn new() -> Self {
        Self {
            printers: DashSet::new(),
        }
    }
}

#[async_trait::async_trait]
impl NetworkPrinter for BambuX1Carbon {
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
            let mut ip = None;
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

            // At this point, we have a valid (as long as the parsing above is strict enough lmao)
            // collection of data that represents a Bambu X1 Carbon.
            let info = NetworkPrinterInfo {
                hostname: name,
                ip,
                port,
                manufacturer: NetworkPrinterManufacturer::Bambu,
                // We can hard code this for now as we check the URN above (and assume the URN is
                // unique to the X1 carbon)
                model: Some(String::from("Bambu Lab X1 Carbon")),
            };

            if self.printers.insert(info.clone()) {
                tracing::info!("Found printer {:?} at IP {:?}", info.hostname, info.ip);
            }
        }

        Ok(())
    }

    fn list(&self) -> anyhow::Result<Vec<NetworkPrinterInfo>> {
        Ok(self.printers.iter().map(|item| item.clone()).collect())
    }
}
