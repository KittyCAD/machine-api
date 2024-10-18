use anyhow::Result;
use bambulabs::{client::Client, command::Command};

use super::{Bambu, PrinterInfo};
use crate::{
    traits::Filament, Control as ControlTrait, FdmHardwareConfiguration, FilamentMaterial, HardwareConfiguration,
    MachineInfo as MachineInfoTrait, MachineMakeModel, MachineState, MachineType,
    SuspendControl as SuspendControlTrait, ThreeMfControl as ThreeMfControlTrait, ThreeMfTemporaryFile, Volume,
};

impl Bambu {
    /// Return a borrow of the underlying Client.
    pub fn inner(&self) -> &Client {
        self.client.as_ref()
    }

    /// Get the latest status of the printer.
    pub fn get_status(&self) -> Result<Option<bambulabs::message::PushStatus>> {
        self.client.get_status()
    }

    /// Check if the printer has an AMS.
    pub fn has_ams(&self) -> Result<bool> {
        let Some(status) = self.get_status()? else {
            return Ok(false);
        };

        let Some(ams) = status.ams else {
            return Ok(false);
        };

        let Some(ams_exists) = ams.ams_exist_bits else {
            return Ok(false);
        };

        Ok(ams_exists != "0")
    }
}

impl MachineInfoTrait for PrinterInfo {
    fn machine_type(&self) -> MachineType {
        MachineType::FusedDeposition
    }

    fn make_model(&self) -> MachineMakeModel {
        self.make_model.clone()
    }

    fn max_part_volume(&self) -> Option<Volume> {
        Some(Volume {
            width: 256.0,
            height: 256.0,
            depth: 256.0,
        })
    }
}
impl ControlTrait for Bambu {
    type Error = anyhow::Error;
    type MachineInfo = PrinterInfo;

    async fn machine_info(&self) -> Result<PrinterInfo> {
        Ok(self.info.clone())
    }

    async fn emergency_stop(&mut self) -> Result<()> {
        self.stop().await
    }

    async fn stop(&mut self) -> Result<()> {
        self.client.publish(Command::stop()).await?;
        Ok(())
    }

    async fn progress(&self) -> Result<Option<f64>> {
        let Some(status) = self.get_status()? else {
            return Ok(None);
        };
        Ok(status.mc_percent.map(|v| v as f64))
    }

    async fn healthy(&self) -> bool {
        let Ok(Some(status)) = self.client.get_status() else {
            return false;
        };

        status.online.is_some()
    }

    async fn state(&self) -> Result<MachineState> {
        let Some(status) = self.client.get_status()? else {
            return Ok(MachineState::Unknown);
        };

        let Some(state) = status.gcode_state else {
            return Ok(MachineState::Unknown);
        };

        match state {
            bambulabs::message::GcodeState::Idle
            | bambulabs::message::GcodeState::Finish
            | bambulabs::message::GcodeState::Failed => Ok(MachineState::Idle),
            bambulabs::message::GcodeState::Running | bambulabs::message::GcodeState::Prepare => {
                Ok(MachineState::Running)
            }
            bambulabs::message::GcodeState::Pause => Ok(MachineState::Paused),
        }
    }

    /// Return the information for the machine for the slicer.
    async fn hardware_configuration(&self) -> Result<HardwareConfiguration> {
        let Some(status) = self.client.get_status()? else {
            anyhow::bail!("Failed to get status");
        };

        let default = HardwareConfiguration::Fdm {
            config: FdmHardwareConfiguration {
                nozzle_diameter: status.nozzle_diameter.into(),
                filaments: vec![Filament {
                    material: FilamentMaterial::Pla,
                    ..Default::default()
                }],
                loaded_filament_idx: None,
            },
        };

        let Some(nams) = status.ams else {
            return Ok(default);
        };

        let Some(ams) = nams.ams.first() else {
            return Ok(default);
        };

        let mut filaments = vec![];
        for tray in &ams.tray {
            let f = Filament {
                material: match tray.tray_type.as_deref() {
                    Some("PLA") => FilamentMaterial::Pla,
                    Some("PLA-S") => FilamentMaterial::PlaSupport,
                    Some("ABS") => FilamentMaterial::Abs,
                    Some("PETG") => FilamentMaterial::Petg,
                    Some("TPU") => FilamentMaterial::Tpu,
                    Some("PVA") => FilamentMaterial::Pva,
                    Some("HIPS") => FilamentMaterial::Hips,
                    _ => {
                        tracing::warn!("Unknown filament type: {:?}", tray.tray_type);
                        FilamentMaterial::Pla
                    }
                },
                name: tray.tray_sub_brands.clone(),
                color: tray.tray_color.clone(),
            };

            filaments.push(f);
        }

        Ok(HardwareConfiguration::Fdm {
            config: FdmHardwareConfiguration {
                nozzle_diameter: status.nozzle_diameter.into(),
                filaments,
                loaded_filament_idx: nams.tray_now.map(|v| v.parse().unwrap_or(0)),
            },
        })
    }
}

impl SuspendControlTrait for Bambu {
    async fn pause(&mut self) -> Result<()> {
        self.client.publish(Command::pause()).await?;
        Ok(())
    }

    async fn resume(&mut self) -> Result<()> {
        self.client.publish(Command::resume()).await?;
        Ok(())
    }
}

impl ThreeMfControlTrait for Bambu {
    async fn build(&mut self, job_name: &str, gcode: ThreeMfTemporaryFile) -> Result<()> {
        let gcode = gcode.0;

        // Upload the file to the printer.
        self.client.upload_file(gcode.path()).await?;

        // Get just the filename.
        let filename = gcode
            .path()
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("No filename: {}", gcode.path().display()))?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Bad filename: {}", gcode.path().display()))?;

        // Check if the printer has an AMS.
        let has_ams = self.has_ams()?;

        self.client
            .publish(Command::print_file(job_name, filename, has_ams))
            .await?;

        Ok(())
    }
}
