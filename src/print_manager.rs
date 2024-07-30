use crate::{gcode::GcodeSequence, server::endpoints::PrintParameters, usb_printer::UsbPrinter};

pub struct PrintJob {
    gcode: GcodeSequence,
    params: PrintParameters,
}

impl PrintJob {
    pub fn new(gcode: GcodeSequence, params: PrintParameters) -> Self {
        Self { gcode, params }
    }

    pub async fn spawn(&self) -> tokio::task::JoinHandle<anyhow::Result<()>> {
        let printer_id = self.params.printer_id.clone();
        let gcode = self.gcode.clone();
        tokio::task::spawn_blocking(move || {
            let printers = UsbPrinter::list_all();
            let printer = printers
                .find_by_id(printer_id.to_owned())
                .expect("Printer not found by given ID");
            let mut printer = UsbPrinter::new(printer);
            printer.wait_for_start()?;

            for line in gcode.lines.iter() {
                let msg = format!("{}\r\n", line);
                println!("writing: {}", line);
                printer.writer.write_all(msg.as_bytes())?;
                printer.wait_for_ok()?;
            }
            Ok(())
        })
    }
}
