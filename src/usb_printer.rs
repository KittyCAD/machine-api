use std::io::BufRead;

pub struct UsbPrinter {
    pub reader: std::io::BufReader<Box<dyn serialport::SerialPort>>,
    pub writer: Box<dyn serialport::SerialPort>,
}

impl UsbPrinter {
    pub fn new() -> Self {
        let port = serialport::new("/dev/ttyACM0", 115_200)
            .timeout(std::time::Duration::from_millis(5000))
            .open()
            .expect("Failed to open port");
        let reader = std::io::BufReader::new(port.try_clone().expect("Split reader"));

        Self { reader, writer: port }
    }

    pub fn wait_for_start(&mut self) -> anyhow::Result<()> {
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

    pub fn wait_for_ok(&mut self) -> anyhow::Result<()> {
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
}
