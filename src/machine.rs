use crate::{AnyMachine, AnySlicer, ControlGcode, DesignFile, Slicer};
use anyhow::Result;

/// Create a handle to a specific Machine which is capable of producing a 3D
/// object in the real world from a specific [crate::DesignFile].
pub struct Machine {
    machine: AnyMachine,
    slicer: AnySlicer,
}

impl Machine {
    /// Take a specific [DesignFile], and produce a real-world 3D object
    /// from it.
    pub async fn print(&self, job_name: &str, design_file: &DesignFile) -> Result<()> {
        // TODO: this only supports gcode for now. This may need to be
        // restructured later.
        let gcode = self.slicer.generate(design_file).await?;

        // TODO: this only supports gcode via the ControlGcode trait. As a
        // result, this match serves the purpose of figuring out what
        // technique we should use.
        match &self.machine {
            AnyMachine::BambuX1Carbon(machine) => machine.build(job_name, gcode).await,
            AnyMachine::Moonraker(machine) => machine.build(job_name, gcode).await,
            AnyMachine::Usb(machine) => machine.build(job_name, gcode).await,
        }
    }
}
