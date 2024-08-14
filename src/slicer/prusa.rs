//! Support for the Prusa Slicer (https://github.com/prusa3d/PrusaSlicer/),
//! which is based on slic3r.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::process::Command;

use crate::{DesignFile, GcodeSlicer as GcodeSlicerTrait, TemporaryFile};

/// Handle to invoke the Prusa Slicer with some specific machine-specific config.
pub struct Slicer {
    config: PathBuf,
}

impl Slicer {
    /// Create a new [Slicer], which will invoke the Prusa Slicer binary
    /// with the specified configuration file.
    pub fn new(config: &Path) -> Self {
        tracing::debug!(config = config.to_str(), "new");
        Self {
            config: config.to_owned(),
        }
    }
}

impl GcodeSlicerTrait for Slicer {
    type Error = anyhow::Error;

    /// Generate gcode from some input file.
    async fn generate(&self, design_file: &DesignFile) -> Result<TemporaryFile> {
        // TODO: support 3mf and other export targets through new traits.

        let uid = uuid::Uuid::new_v4();
        let gcode_path = std::env::temp_dir().join(format!("{}.gcode", uid.simple()));

        let (file_path, file_type) = match design_file {
            DesignFile::Stl(path) => (path, "stl"),
        };

        tracing::info!(
            config = self.config.to_str(),
            file_path = file_path.to_str(),
            file_type = file_type,
            "building to gcode"
        );

        let args: Vec<String> = vec![
            "--load".to_string(),
            self.config
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid slicer config path: {}", self.config.display()))?
                .to_string(),
            "--support-material".to_string(),
            "--export-gcode".to_string(),
            file_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid original file path: {}", file_path.display()))?
                .to_string(),
            "--output".to_string(),
            gcode_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid output G-code path: {}", gcode_path.display()))?
                .to_string(),
        ];

        let output = Command::new(find_prusa_slicer()?)
            .args(&args)
            .output()
            .await
            .context("Failed to execute prusa-slicer command")?;

        // Make sure the command was successful.
        if !output.status.success() {
            let stdout = std::str::from_utf8(&output.stdout)?;
            let stderr = std::str::from_utf8(&output.stderr)?;

            tracing::warn!(
                config = self.config.to_str(),
                file_path = file_path.to_str(),
                file_type = file_type,
                "failed to build gcode",
            );

            anyhow::bail!("Failed to : {:?}\nstdout:\n{}stderr:{}", output, stdout, stderr);
        }

        // Make sure the G-code file was created.
        if !gcode_path.exists() {
            anyhow::bail!("Failed to create G-code file");
        }

        tracing::info!(
            config = self.config.to_str(),
            file_path = file_path.to_str(),
            file_type = file_type,
            gcode_path = gcode_path.to_str(),
            "gcode built",
        );

        TemporaryFile::new(&gcode_path).await
    }
}

// Find the prusaslicer executable path on macOS.
#[cfg(target_os = "macos")]
fn find_prusa_slicer() -> Result<PathBuf> {
    let app_path = PathBuf::from("/Applications/Slicer.app/Contents/MacOS/Slicer");
    if app_path.exists() {
        Ok(app_path)
    } else {
        anyhow::bail!("Slicer not found")
    }
}

// Find the prusaslicer executable path on Windows.
#[cfg(target_os = "windows")]
fn find_prusa_slicer() -> Result<PathBuf> {
    let app_path = PathBuf::from("C:\\Program Files\\Slicer\\Slicer.exe");
    if app_path.exists() {
        Ok(app_path)
    } else {
        anyhow::bail!("Slicer not found")
    }
}

// Find the prusaslicer executable path on Linux.
#[cfg(target_os = "linux")]
fn find_prusa_slicer() -> Result<PathBuf> {
    let app_path = PathBuf::from("/usr/bin/prusa-slicer");
    if app_path.exists() {
        Ok(app_path)
    } else {
        anyhow::bail!("Slicer not found")
    }
}
