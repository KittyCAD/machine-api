use crate::gcode::Client;
use crate::{
    Control as ControlTrait, GcodeControl as GcodeControlTrait, GcodeTemporaryFile, MachineInfo as MachineInfoTrait,
    MachineMakeModel, MachineType, Volume,
};
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio_serial::SerialStream;

/// Handle to a USB based gcode 3D printer.
pub struct Usb {
    client: Client<WriteHalf<SerialStream>, ReadHalf<SerialStream>>,
    machine_info: UsbMachineInfo,
}

impl Usb {
    /// Create a new USB-based gcode Machine.
    pub fn new(stream: SerialStream, machine_info: UsbMachineInfo) -> Self {
        let (reader, writer) = tokio::io::split(stream);

        Self {
            client: Client::new(writer, reader),
            machine_info,
        }
    }

    async fn wait_for_start(&mut self) -> Result<()> {
        loop {
            let mut line = String::new();
            if let Err(e) = self.client.get_read().read_line(&mut line).await {
                println!("wait_for_start err: {}", e);
            } else {
                // Use ends with because sometimes we may still have some data left on the buffer
                if line.trim().ends_with("start") {
                    return Ok(());
                }
            }
        }
    }

    async fn wait_for_ok(&mut self) -> Result<()> {
        loop {
            let mut line = String::new();
            if let Err(e) = self.client.get_read().read_line(&mut line).await {
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
#[derive(Clone, Debug, PartialEq)]
pub struct UsbMachineInfo {
    machine_type: MachineType,
    make_model: MachineMakeModel,
    volume: Option<Volume>,

    /// USB Vendor ID
    pub vendor_id: u16,

    /// USB Product ID
    pub product_id: u16,

    /// USB Port (/dev/ttyUSB0, etc).
    pub port: String,

    /// Baud rate of the Serial connection.
    pub baud: u32,
}

impl UsbMachineInfo {
    /// Create a new USB Machine Info directly (not via discovery).
    pub fn new(
        machine_type: MachineType,
        make_model: MachineMakeModel,
        volume: Option<Volume>,
        vendor_id: u16,
        product_id: u16,
        port: String,
        baud: u32,
    ) -> Self {
        Self {
            machine_type,
            make_model,
            volume,
            vendor_id,
            product_id,
            port,
            baud,
        }
    }

    // /// return the discovery key
    // pub(crate) fn discovery_key(&self) -> (u16, u16, String) {
    //     (self.vendor_id, self.product_id, self.make_model.serial.clone().unwrap())
    // }
}

impl MachineInfoTrait for UsbMachineInfo {
    fn machine_type(&self) -> MachineType {
        self.machine_type
    }

    fn make_model(&self) -> MachineMakeModel {
        self.make_model.clone()
    }

    fn max_part_volume(&self) -> Option<Volume> {
        self.volume
    }
}

impl ControlTrait for Usb {
    type MachineInfo = UsbMachineInfo;
    type Error = anyhow::Error;

    async fn machine_info(&self) -> Result<UsbMachineInfo> {
        Ok(self.machine_info.clone())
    }

    async fn emergency_stop(&mut self) -> Result<()> {
        self.client.emergency_stop().await
    }

    async fn stop(&mut self) -> Result<()> {
        self.client.stop().await
    }

    async fn healthy(&self) -> bool {
        // TODO: fix this, do a gcode ping or something?
        true
    }
}

impl GcodeControlTrait for Usb {
    async fn build(&mut self, _job_name: &str, gcode: GcodeTemporaryFile) -> Result<()> {
        let mut gcode = gcode.0;

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
            self.client.write_all(msg.as_bytes()).await?;
            self.wait_for_ok().await?;
        }

        Ok(())
    }
}