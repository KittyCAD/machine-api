use crate::{AnyMachine, AnySlicer, ControlGcode, DesignFile, Slicer};
use anyhow::Result;

/// Create a handle to a specific Machine which is capable of producing a 3D
/// object in the real world from a specific [crate::DesignFile].
pub struct Machine {
    machine: AnyMachine,
    slicer: AnySlicer,
}

impl Machine {
    /// Create a new [Machine] from a specific [AnyMachine] control channel,
    /// and a specific [AnySlicer] slicer.
    pub fn new<MachineT, SlicerT>(machine: MachineT, slicer: SlicerT) -> Self
    where
        MachineT: Into<AnyMachine>,
        SlicerT: Into<AnySlicer>,
    {
        Self {
            machine: machine.into(),
            slicer: slicer.into(),
        }
    }

    /// Return the underlying [AnyMachine] enum.
    pub fn get_machine(&self) -> &AnyMachine {
        &self.machine
    }

    /// Return the underlying [AnyMachine] enum as a mutable borrow.
    pub fn get_machine_mut(&mut self) -> &mut AnyMachine {
        &mut self.machine
    }

    /// Return the underlying [AnySlicer] enum.
    pub fn get_slicer(&self) -> &AnySlicer {
        &self.slicer
    }

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
