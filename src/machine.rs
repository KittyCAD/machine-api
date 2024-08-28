use crate::{AnyMachine, AnySlicer, DesignFile, GcodeControl, GcodeSlicer, ThreeMfControl, ThreeMfSlicer};
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

    /// Return the underlying [AnySlicer] enum as a mutable borrow.
    pub fn get_slicer_mut(&mut self) -> &mut AnySlicer {
        &mut self.slicer
    }

    /// Take a specific [DesignFile], and produce a real-world 3D object
    /// from it.
    pub async fn build(&mut self, job_name: &str, design_file: &DesignFile) -> Result<()> {
        tracing::debug!(name = job_name, "building");

        match &mut self.machine {
            AnyMachine::BambuX1Carbon(machine) => {
                let three_mf = ThreeMfSlicer::generate(&self.slicer, design_file).await?;
                ThreeMfControl::build(machine, job_name, three_mf).await
            }
            AnyMachine::Moonraker(machine) => {
                let gcode = GcodeSlicer::generate(&self.slicer, design_file).await?;
                GcodeControl::build(machine, job_name, gcode).await
            }
            AnyMachine::Usb(machine) => {
                let gcode = GcodeSlicer::generate(&self.slicer, design_file).await?;
                GcodeControl::build(machine, job_name, gcode).await
            }
            AnyMachine::Noop(_) => {
                // why even bother ;)
                Ok(())
            }
        }
    }
}
