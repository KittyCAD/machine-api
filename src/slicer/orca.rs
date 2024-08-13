//! Support for the orca Slicer.

use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::process::Command;

use crate::{DesignFile, TemporaryFile};

/// Handle to invoke the Orca Slicer with some specific machine-specific config.
pub struct Slicer {
    config: PathBuf,
}

impl Slicer {
    /// Create a new [Slicer], which will invoke the Orca Slicer binary
    /// with the specified configuration file.
    pub fn new(config: PathBuf) -> Self {
        Self { config }
    }
}

impl Slicer {
    /// Generate gcode from some input file.
    pub async fn generate(&self, design_file: &DesignFile) -> Result<TemporaryFile> {
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
        let gcode_path = std::env::temp_dir().join(format!("{}.3mf", uid));
        let process_config = self
            .config
            .join("process.json")
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid slicer config path: {}", self.config.display()))?
            .to_string();
        let machine_config = self
            .config
            .join("machine.json")
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid slicer config path: {}", self.config.display()))?
            .to_string();
        let filament_config = self
            .config
            .join("filament.json")
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
            "--export-3mf".to_string(),
            gcode_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid output G-code path: {}", gcode_path.display()))?
                .to_string(),
            file_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid original file path: {}", file_path.display()))?
                .to_string(),
        ];

        // Find the orcaslicer executable path.
        let orca_slicer_path = find_orca_slicer()?;

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
        if !gcode_path.exists() {
            anyhow::bail!("Failed to create G-code file");
        }

        TemporaryFile::new(&gcode_path).await
    }
}

// Find the orcaslicer executable path on macOS.
#[cfg(target_os = "macos")]
fn find_orca_slicer() -> Result<PathBuf> {
    let app_path = PathBuf::from("/Applications/Slicer.app/Contents/MacOS/Slicer");
    if app_path.exists() {
        Ok(app_path)
    } else {
        anyhow::bail!("Slicer not found")
    }
}

// Find the orcaslicer executable path on Windows.
#[cfg(target_os = "windows")]
fn find_orca_slicer() -> Result<PathBuf> {
    let app_path = PathBuf::from("C:\\Program Files\\Slicer\\orca-slicer.exe");
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
