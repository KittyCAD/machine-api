use std::sync::Arc;

use anyhow::Result;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    sync::Mutex,
};
use tokio_serial::SerialStream;

use crate::{
    gcode::Client, traits::Filament, Control as ControlTrait, FdmHardwareConfiguration, FilamentMaterial,
    GcodeControl as GcodeControlTrait, GcodeTemporaryFile, HardwareConfiguration, MachineInfo as MachineInfoTrait,
    MachineMakeModel, MachineState, MachineType, Volume,
};

/// Handle to a USB based gcode 3D printer.
#[derive(Clone)]
pub struct Usb {
    client: Arc<Mutex<Client<WriteHalf<SerialStream>, ReadHalf<SerialStream>>>>,
    machine_info: UsbMachineInfo,
}

impl Usb {
    /// Create a new USB-based gcode Machine.
    pub fn new(stream: SerialStream, machine_info: UsbMachineInfo) -> Self {
        let (reader, writer) = tokio::io::split(stream);

        Self {
            client: Arc::new(Mutex::new(Client::new(writer, reader))),
            machine_info,
        }
    }

    async fn wait_for_start(&mut self) -> Result<()> {
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

    async fn wait_for_ok(&mut self) -> Result<()> {
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
        self.client.lock().await.emergency_stop().await
    }

    async fn stop(&mut self) -> Result<()> {
        self.client.lock().await.stop().await
    }

    async fn state(&self) -> Result<MachineState> {
        Ok(MachineState::Unknown)
    }

    async fn progress(&self) -> Result<Option<f64>> {
        Ok(None)
    }

    async fn healthy(&self) -> bool {
        // TODO: fix this, do a gcode ping or something?
        true
    }

    async fn hardware_configuration(&self) -> Result<HardwareConfiguration> {
        Ok(HardwareConfiguration::Fdm {
            config: FdmHardwareConfiguration {
                filaments: vec![Filament {
                    material: FilamentMaterial::Pla,
                    ..Default::default()
                }],
                nozzle_diameter: 0.4,
                loaded_filament_idx: None,
            },
        })
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

        let self1 = self.clone();
        tokio::spawn(async move {
            let mut usb = self1;
            for line in lines.iter() {
                let msg = format!("{}\r\n", line);
                println!("writing: {}", line);
                usb.client.lock().await.write_all(msg.as_bytes()).await?;
                usb.wait_for_ok().await?;
            }
            Ok::<(), anyhow::Error>(())
        });

        // store a handle to the joinhandle in an option in self or something?

        Ok(())
    }
}
