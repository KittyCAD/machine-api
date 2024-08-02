use std::{collections::HashMap, io::BufRead, path::PathBuf};

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serialport::SerialPortType;

use crate::gcode::GcodeFile;

pub struct UsbPrinter {
    pub reader: std::io::BufReader<Box<dyn serialport::SerialPort>>,
    pub writer: Box<dyn serialport::SerialPort>,
}

/// Details for a 3d printer connected over USB.
#[derive(Clone, Debug, JsonSchema, Serialize, Deserialize)]
pub struct UsbPrinterInfo {
    pub port: String,
    pub id: String,
    pub manufacturer: String,
    pub model: String,
}

impl UsbPrinter {
    // List all 3d printers connected over USB.
    pub fn list_all() -> HashMap<String, UsbPrinterInfo> {
        let list: Vec<UsbPrinterInfo> = serialport::available_ports()
            .expect("No ports found!")
            .iter()
            .filter_map(|p| {
                if let SerialPortType::UsbPort(usb) = p.port_type.clone() {
                    Some(UsbPrinterInfo {
                        port: p.port_name.clone(),
                        id: usb.serial_number.unwrap_or("Unknown".to_string()),
                        manufacturer: usb.manufacturer.unwrap_or("Unknown manufacturer".to_string()),
                        model: usb.product.unwrap_or("Unknown product".to_string()),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        list.into_iter().map(|p| (p.id.clone(), p)).collect()
    }

    pub fn new(printer: UsbPrinterInfo) -> Self {
        let port = serialport::new(printer.port, 115_200)
            .timeout(std::time::Duration::from_millis(5000))
            .open()
            .expect("Failed to open port");
        let reader = std::io::BufReader::new(port.try_clone().expect("Split reader"));

        Self { reader, writer: port }
    }

    pub fn wait_for_start(&mut self) -> Result<()> {
        loop {
            let mut line = String::new();
            if let Err(e) = self.reader.read_line(&mut line) {
                println!("wait_for_start err: {}", e);
            } else {
                // Use ends with because sometimes we may still have some data left on the buffer
                if line.trim().ends_with("start") {
                    return Ok(());
                }
            }
        }
    }

    pub fn wait_for_ok(&mut self) -> Result<()> {
        loop {
            let mut line = String::new();
            if let Err(e) = self.reader.read_line(&mut line) {
                println!("wait_for_ok err: {}", e);
            } else {
                println!("RCVD: {}", line);
                if line.trim() == "ok" {
                    return Ok(());
                }
            }
        }
    }

    fn slice(&self, file: &std::path::Path) -> Result<PathBuf> {
        // TODO: make this a configurable path.
        let gcode = GcodeFile::from_stl_path(
            crate::gcode::Slicer::Prusa,
            std::path::Path::new("../config/prusa/mk3.ini"),
            file,
        )?;

        Ok(gcode.path)
    }

    fn print(&mut self, file: &std::path::Path) -> Result<()> {
        // Read the gcode file.
        let lines: Vec<String> = std::fs::read_to_string(file)?
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

        self.wait_for_start()?;

        for line in lines.iter() {
            let msg = format!("{}\r\n", line);
            println!("writing: {}", line);
            self.writer.write_all(msg.as_bytes())?;
            self.wait_for_ok()?;
        }

        Ok(())
    }

    pub fn slice_and_print(&mut self, file: &std::path::Path) -> Result<()> {
        let gcode = self.slice(file)?;
        self.print(&gcode)
    }
}
