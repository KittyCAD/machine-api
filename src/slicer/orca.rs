//! Support for the orca Slicer.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tokio::process::Command;

use crate::{
    DesignFile, HardwareConfiguration, SlicerConfiguration, TemporaryFile, ThreeMfSlicer as ThreeMfSlicerTrait,
    ThreeMfTemporaryFile,
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
        _slicer_configuration: &SlicerConfiguration,
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

        let uid = uuid::Uuid::new_v4();
        let output_path = std::env::temp_dir().join(format!("{}.{}", uid, output_extension));
        let process_p = self
            .config
            .join("process.json")
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid slicer config path: {}", self.config.display()))?
            .to_string();
        let process_str = tokio::fs::read_to_string(&process_p).await?;
        let mut process_overrides: bambulabs::templates::Template = serde_json::from_str(&process_str)?;
        let machine_p = self
            .config
            .join("machine.json")
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid slicer config path: {}", self.config.display()))?
            .to_string();
        let machine_str = tokio::fs::read_to_string(&machine_p).await?;
        let mut machine_overrides: bambulabs::templates::Template = serde_json::from_str(&machine_str)?;
        let filament_p = self
            .config
            .join("filament.json")
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid slicer config path: {}", self.config.display()))?
            .to_string();
        let filament_str = tokio::fs::read_to_string(&filament_p).await?;
        let mut filament_overrides: bambulabs::templates::Template = serde_json::from_str(&filament_str)?;

        if let HardwareConfiguration::Fdm(fdm) = hardware_configuration {
            // TODO: we should populate for other bambu printers.
            // TODO: set the right filament.
            match fdm.nozzle_diameter {
                0.2 => {
                    // Merge the bambu templates.
                    process_overrides.set_inherits("0.10mm Standard @BBL X1C");
                    machine_overrides.set_inherits("Bambu Lab X1 Carbon 0.2 nozzle");
                    filament_overrides.set_inherits("Bambu PLA Basic @BBL X1C 0.2 nozzle");
                }
                0.4 => {
                    // Merge the bambu templates.
                    process_overrides.set_inherits("0.20mm Standard @BBL X1C");
                    machine_overrides.set_inherits("Bambu Lab X1 Carbon 0.4 nozzle");
                    filament_overrides.set_inherits("Bambu PLA Basic @BBL X1C");
                }
                0.6 => {
                    // Merge the bambu templates.
                    process_overrides.set_inherits("0.30mm Standard @BBL X1C 0.6 nozzle");
                    machine_overrides.set_inherits("Bambu Lab X1 Carbon 0.6 nozzle");
                    filament_overrides.set_inherits("Bambu PLA Basic @BBL X1C");
                }
                0.8 => {
                    // Merge the bambu templates.
                    process_overrides.set_inherits("0.40mm Standard @BBL X1C 0.8 nozzle");
                    machine_overrides.set_inherits("Bambu Lab X1 Carbon 0.8 nozzle");
                    filament_overrides.set_inherits("Bambu PLA Basic @BBL X1C 0.8 nozzle");
                }
                other => anyhow::bail!("Unsupported nozzle diameter for orca: {}", other),
            }
        } else {
            anyhow::bail!("Unsupported hardware configuration for orca");
        }

        // Traverse the templates and merge them.
        let new_process = process_overrides.load_inherited()?;
        let new_machine = machine_overrides.load_inherited()?;
        let new_filament = filament_overrides.load_inherited()?;

        // Write each to a temporary file.
        let temp_dir = std::env::temp_dir();
        let process_config = temp_dir.join(format!("process-{}.json", uid));
        tokio::fs::write(&process_config, serde_json::to_string_pretty(&new_process)?).await?;
        let machine_config = temp_dir.join(format!("machine-{}.json", uid));
        tokio::fs::write(&machine_config, serde_json::to_string_pretty(&new_machine)?).await?;
        let filament_config = temp_dir.join(format!("filament-{}.json", uid));
        tokio::fs::write(&filament_config, serde_json::to_string_pretty(&new_filament)?).await?;
        let filament_config = filament_config
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid filament config path: {}", filament_config.display()))?
            .to_string();
        let process_config = process_config
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid process config path: {}", process_config.display()))?
            .to_string();
        let machine_config = machine_config
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid machine config path: {}", machine_config.display()))?
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
        slicer_configuration: &SlicerConfiguration,
    ) -> Result<ThreeMfTemporaryFile> {
        Ok(ThreeMfTemporaryFile(
            self.generate_via_cli(
                "--export-3mf",
                "3mf",
                design_file,
                hardware_configuration,
                slicer_configuration,
            )
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
