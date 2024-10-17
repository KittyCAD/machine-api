//! Support for the orca Slicer.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tokio::process::Command;

use crate::{
    DesignFile, HardwareConfiguration, TemporaryFile, ThreeMfSlicer as ThreeMfSlicerTrait, ThreeMfTemporaryFile,
};

/// Handle to invoke the Orca Slicer with some specific machine-specific config.
pub struct Slicer {
    config: PathBuf,
}

impl Slicer {
    /// Create a new [Slicer], which will invoke the Orca Slicer binary
    /// with the specified configuration file.
    pub fn new(config: &Path) -> Self {
        Self {
            config: config.to_path_buf(),
        }
    }

    /// Generate 3MF from some input file.
    async fn generate_via_cli(
        &self,
        output_flag: &str,
        output_extension: &str,
        design_file: &DesignFile,
        hardware_configuration: &HardwareConfiguration,
    ) -> Result<TemporaryFile> {
        // Make sure the config path is a directory.
        if !self.config.is_dir() {
            anyhow::bail!(
                "Invalid slicer config path: {}, must be a directory",
                self.config.display()
            );
        }

        let (file_path, _file_type) = match design_file {
            DesignFile::Stl(path) => (path, "stl"),
        };

        let (process_file, machine_file, filament_file) = match machine_info.nozzle_diameter {
            bambulabs::message::NozzleDiameter::Diameter02 => (
                "process-0.10mm.json",
                "machine-0.2-nozzle.json",
                "filament-0.2-nozzle.json",
            ),
            bambulabs::message::NozzleDiameter::Diameter04 => {
                ("process-0.20mm.json", "machine-0.4-nozzle.json", "filament.json")
            }
            // TODO: Add support for these nozzles and better template them.
            bambulabs::message::NozzleDiameter::Diameter06 => anyhow::bail!("No configuration for 0.6mm nozzle"),
            bambulabs::message::NozzleDiameter::Diameter08 => anyhow::bail!("No configuration for 0.8mm nozzle"),
        };

        let uid = uuid::Uuid::new_v4();
        let output_path = std::env::temp_dir().join(format!("{}.{}", uid, output_extension));
        let process_config = self
            .config
            .join(process_file)
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid slicer config path: {}", self.config.display()))?
            .to_string();
        let machine_config = self
            .config
            .join(machine_file)
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid slicer config path: {}", self.config.display()))?
            .to_string();
        let filament_config = self
            .config
            .join(filament_file)
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid slicer config path: {}", self.config.display()))?
            .to_string();

        let settings = [process_config, machine_config].join(";");

        let args: Vec<String> = vec![
            "--load-settings".to_string(),
            settings,
            "--load-filaments".to_string(),
            filament_config,
            "--slice".to_string(),
            "0".to_string(),
            "--orient".to_string(),
            "1".to_string(),
            output_flag.to_string(),
            output_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid slicer output path: {}", output_path.display()))?
                .to_string(),
            file_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid original file path: {}", file_path.display()))?
                .to_string(),
        ];

        // Find the orcaslicer executable path.
        let orca_slicer_path = find_orca_slicer()?;

        println!("orca_slicer_path: {:?}", orca_slicer_path);
        println!("args: {:?}", args);

        let output = Command::new(orca_slicer_path)
            .args(&args)
            .output()
            .await
            .context("Failed to execute orca-slicer command")?;

        // Make sure the command was successful.
        if !output.status.success() {
            let stdout = std::str::from_utf8(&output.stdout)?;
            let stderr = std::str::from_utf8(&output.stderr)?;
            anyhow::bail!("Failed to : {:?}\nstdout:\n{}stderr:{}", output, stdout, stderr);
        }

        // Make sure the G-code file was created.
        if !output_path.exists() {
            anyhow::bail!("Failed to create output file");
        }

        TemporaryFile::new(&output_path).await
    }
}

impl ThreeMfSlicerTrait for Slicer {
    type Error = anyhow::Error;

    /// Generate gcode from some input file.
    async fn generate(
        &self,
        design_file: &DesignFile,
        hardware_configuration: &HardwareConfiguration,
    ) -> Result<ThreeMfTemporaryFile> {
        Ok(ThreeMfTemporaryFile(
            self.generate_via_cli("--export-3mf", "3mf", design_file, hardware_configuration)
                .await?,
        ))
    }
}

// Find the orcaslicer executable path on macOS.
#[cfg(target_os = "macos")]
fn find_orca_slicer() -> Result<PathBuf> {
    let app_path = PathBuf::from("/Applications/OrcaSlicer.app/Contents/MacOS/OrcaSlicer");
    if app_path.exists() {
        Ok(app_path)
    } else {
        anyhow::bail!("Slicer not found")
    }
}

// Find the orcaslicer executable path on Windows.
#[cfg(target_os = "windows")]
fn find_orca_slicer() -> Result<PathBuf> {
    let app_path = PathBuf::from("C:\\Program Files\\OrcaSlicer\\orca-slicer.exe");
    if app_path.exists() {
        Ok(app_path)
    } else {
        anyhow::bail!("Slicer not found")
    }
}

// Find the orcaslicer executable path on Linux.
#[cfg(target_os = "linux")]
fn find_orca_slicer() -> Result<PathBuf> {
    let app_path = PathBuf::from("/usr/bin/orca-slicer");
    if app_path.exists() {
        Ok(app_path)
    } else {
        anyhow::bail!("Slicer not found")
    }
}
