use super::*;
use crate::{
    Control as ControlTrait, GcodeControl as GcodeControlTrait, GcodeTemporaryFile, MachineInfo as MachineInfoTrait,
    MachineMakeModel, MachineType, Volume,
};
use tokio::io::{ReadHalf, WriteHalf};
use tokio_serial::SerialStream;

///
pub struct Usb {
    client: Client<WriteHalf<SerialStream>, ReadHalf<SerialStream>>,

    machine_type: MachineType,
    volume: Volume,
    make_model: MachineMakeModel,
}

impl Usb {
    /// Create a new USB-based gcode Machine.
    pub fn new(stream: SerialStream, machine_type: MachineType, volume: Volume, make_model: MachineMakeModel) -> Self {
        let (reader, writer) = tokio::io::split(stream);

        Self {
            client: Client::new(writer, reader),
            machine_type,
            volume,
            make_model,
        }
    }

    /// Create a new Machine, talking gcode via USB to a Prusa Research
    /// MK3.
    pub fn prusa_mk3(stream: SerialStream) -> Self {
        Self::new(
            stream,
            MachineType::FusedDeposition,
            Volume {
                width: 250.0,
                depth: 210.0,
                height: 210.0,
            },
            MachineMakeModel {
                manufacturer: Some("Prusa Research".to_owned()),
                model: Some("MK3".to_owned()),
                serial: None,
            },
        )
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
pub struct UsbMachineInfo {
    machine_type: MachineType,
    make_model: MachineMakeModel,
    volume: Volume,
}

impl MachineInfoTrait for UsbMachineInfo {
    fn machine_type(&self) -> MachineType {
        self.machine_type.clone()
    }

    fn make_model(&self) -> MachineMakeModel {
        self.make_model.clone()
    }

    fn max_part_volume(&self) -> Option<Volume> {
        Some(self.volume.clone())
    }
}

impl ControlTrait for Usb {
    type MachineInfo = UsbMachineInfo;
    type Error = anyhow::Error;

    async fn machine_info(&self) -> Result<UsbMachineInfo> {
        Ok(UsbMachineInfo {
            machine_type: self.machine_type.clone(),
            make_model: self.make_model.clone(),
            volume: self.volume.clone(),
        })
    }

    async fn emergency_stop(&mut self) -> Result<()> {
        self.client.emergency_stop().await
    }
    async fn stop(&mut self) -> Result<()> {
        self.client.stop().await
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
