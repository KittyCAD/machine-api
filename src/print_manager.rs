use crate::{gcode::GcodeSequence, machine::Machine, usb_printer::UsbPrinter};

pub struct PrintJob {
    gcode: GcodeSequence,
    machine: Machine,
}

impl PrintJob {
    pub fn new(gcode: GcodeSequence, machine: Machine) -> Self {
        Self { gcode, machine }
    }

    pub async fn spawn(&self) -> tokio::task::JoinHandle<anyhow::Result<()>> {
        let gcode = self.gcode.clone();
        let machine = self.machine.clone();
        tokio::task::spawn_blocking(move || {
            if let Machine::UsbPrinter(printer) = &machine {
                let mut printer = UsbPrinter::new(printer.clone());
                printer.wait_for_start()?;

                for line in gcode.lines.iter() {
                    let msg = format!("{}\r\n", line);
                    println!("writing: {}", line);
                    printer.writer.write_all(msg.as_bytes())?;
                    printer.wait_for_ok()?;
                }

                Ok(())
            } else {
                Err(anyhow::anyhow!("network printers not yet supported"))
            }
        })
    }
}
