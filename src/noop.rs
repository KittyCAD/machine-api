//! `noop` implements a no-op Machine, one that will accept Control commands
//! and do exactly nothing with it.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    Control as ControlTrait, FdmHardwareConfiguration, Filament, GcodeControl as GcodeControlTrait, GcodeTemporaryFile,
    HardwareConfiguration, MachineInfo as MachineInfoTrait, MachineMakeModel, MachineState, MachineType,
    SuspendControl as SuspendControlTrait, ThreeMfControl as ThreeMfControlTrait, ThreeMfTemporaryFile, Volume,
};

/// Noop-machine will no-op, well, everything.
pub struct Noop {
    make_model: MachineMakeModel,
    machine_type: MachineType,
    volume: Option<Volume>,
    config: Config,
}

/// Configuration information for a Moonraker-based endpoint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Extrusion hotend nozzle's diameter.
    pub nozzle_diameter: f64,

    /// Available filaments.
    pub filaments: Vec<Filament>,

    /// Currently loaded filament, if possible to determine.
    pub loaded_filament_idx: Option<usize>,

    /// state that the machine is in
    pub state: MachineState,

    /// percentage through a print
    pub progress: Option<f64>,
}

/// Nothing to see here!
#[derive(Clone, Debug, PartialEq)]
pub struct MachineInfo {
    make_model: MachineMakeModel,
    machine_type: MachineType,
    volume: Option<Volume>,
}

impl MachineInfoTrait for MachineInfo {
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

impl Noop {
    /// Return a new no-op Machine.
    pub fn new(
        config: Config,
        make_model: MachineMakeModel,
        machine_type: MachineType,
        volume: Option<Volume>,
    ) -> Self {
        Self {
            make_model,
            volume,
            machine_type,
            config,
        }
    }
}

impl ControlTrait for Noop {
    type Error = anyhow::Error;
    type MachineInfo = MachineInfo;

    async fn machine_info(&self) -> Result<Self::MachineInfo> {
        Ok(MachineInfo {
            make_model: self.make_model.clone(),
            volume: self.volume,
            machine_type: self.machine_type,
        })
    }

    async fn emergency_stop(&mut self) -> Result<()> {
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    async fn healthy(&self) -> bool {
        true
    }

    async fn progress(&self) -> Result<Option<f64>> {
        Ok(self.config.progress)
    }

    async fn state(&self) -> Result<MachineState> {
        Ok(self.config.state.clone())
    }

    async fn hardware_configuration(&self) -> Result<HardwareConfiguration> {
        let config = &self.config;

        Ok(HardwareConfiguration::Fdm {
            config: FdmHardwareConfiguration {
                filaments: config.filaments.clone(),
                nozzle_diameter: config.nozzle_diameter,
                loaded_filament_idx: config.loaded_filament_idx,
            },
        })
    }
}

impl SuspendControlTrait for Noop {
    async fn pause(&mut self) -> Result<()> {
        Ok(())
    }

    async fn resume(&mut self) -> Result<()> {
        Ok(())
    }
}

impl GcodeControlTrait for Noop {
    async fn build(&mut self, _job_name: &str, _gcode: GcodeTemporaryFile) -> Result<()> {
        Ok(())
    }
}

impl ThreeMfControlTrait for Noop {
    async fn build(&mut self, _job_name: &str, _three_mf: ThreeMfTemporaryFile) -> Result<()> {
        Ok(())
    }
}
