//! Support for the Prusa Slicer (https://github.com/prusa3d/PrusaSlicer/),
//! which is based on slic3r.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tokio::process::Command;

use crate::{
    DesignFile, GcodeSlicer as GcodeSlicerTrait, GcodeTemporaryFile, HardwareConfiguration, TemporaryFile,
    ThreeMfSlicer as ThreeMfSlicerTrait, ThreeMfTemporaryFile,
};

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

    /// Generate gcode from some input file.
    async fn generate_from_cli(
        &self,
        output_flag: &str,
        output_extension: &str,
        design_file: &DesignFile,
    ) -> Result<TemporaryFile> {
        // TODO: support 3mf and other export targets through new traits.

        let uid = uuid::Uuid::new_v4();
        let output_path = std::env::temp_dir().join(format!("{}.{}", uid.simple(), output_extension));

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
            output_flag.to_string(),
            file_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid original file path: {}", file_path.display()))?
                .to_string(),
            "--output".to_string(),
            output_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid output path: {}", output_path.display()))?
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

        // Make sure the file was created.
        if !output_path.exists() {
            anyhow::bail!("Failed to create output file");
        }

        tracing::info!(
            config = self.config.to_str(),
            file_path = file_path.to_str(),
            file_type = file_type,
            output_path = output_path.to_str(),
            "gcode built",
        );

        TemporaryFile::new(&output_path).await
    }
}

impl GcodeSlicerTrait for Slicer {
    type Error = anyhow::Error;

    async fn generate(&self, design_file: &DesignFile, _: &HardwareConfiguration) -> Result<GcodeTemporaryFile> {
        Ok(GcodeTemporaryFile(
            self.generate_from_cli("--export-gcode", "gcode", design_file).await?,
        ))
    }
}

impl ThreeMfSlicerTrait for Slicer {
    type Error = anyhow::Error;

    async fn generate(&self, design_file: &DesignFile, _: &HardwareConfiguration) -> Result<ThreeMfTemporaryFile> {
        Ok(ThreeMfTemporaryFile(
            self.generate_from_cli("--export-3mf", "3mf", design_file).await?,
        ))
    }
}

// Find the prusaslicer executable path on macOS.
#[cfg(target_os = "macos")]
fn find_prusa_slicer() -> Result<PathBuf> {
    let app_path = PathBuf::from("/Applications/PrusaSlicer.app/Contents/MacOS/PrusaSlicer");
    if app_path.exists() {
        Ok(app_path)
    } else {
        anyhow::bail!("Slicer not found")
    }
}

// Find the prusaslicer executable path on Windows.
#[cfg(target_os = "windows")]
fn find_prusa_slicer() -> Result<PathBuf> {
    let app_path = PathBuf::from("C:\\Program Files\\PrusaSlicer\\PrusaSlicer.exe");
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
        // Just assume it's somewhere on the path.
        Ok(PathBuf::from("prusa-slicer"))
    }
}
