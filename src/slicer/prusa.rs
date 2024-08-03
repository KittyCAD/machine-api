use std::path::PathBuf;

use anyhow::{Context, Result};
use tokio::process::Command;

use crate::slicer::Slicer;

pub struct PrusaSlicer {
    config: PathBuf,
}

impl PrusaSlicer {
    pub fn new(config: PathBuf) -> Self {
        Self { config }
    }
}

#[async_trait::async_trait]
impl Slicer for PrusaSlicer {
    async fn slice(&self, file: &std::path::Path) -> Result<std::path::PathBuf> {
        let uid = uuid::Uuid::new_v4();
        let gcode_path = std::env::temp_dir().join(format!("{}.3mf", uid));

        let args: Vec<String> = vec![
            "--load".to_string(),
            self.config
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid slicer config path: {}", self.config.display()))?
                .to_string(),
            "--support-material".to_string(),
            "--export-gcode".to_string(),
            file.to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid original file path: {}", file.display()))?
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
            anyhow::bail!("Failed to : {:?}\nstdout:\n{}stderr:{}", output, stdout, stderr);
        }

        // Make sure the G-code file was created.
        if !gcode_path.exists() {
            anyhow::bail!("Failed to create G-code file");
        }

        Ok(gcode_path.to_path_buf())
    }
}

// Find the prusaslicer executable path on macOS.
#[cfg(target_os = "macos")]
fn find_prusa_slicer() -> anyhow::Result<PathBuf> {
    let app_path = std::path::PathBuf::from("/Applications/PrusaSlicer.app/Contents/MacOS/PrusaSlicer");
    if app_path.exists() {
        Ok(app_path)
    } else {
        anyhow::bail!("PrusaSlicer not found")
    }
}

// Find the prusaslicer executable path on Windows.
#[cfg(target_os = "windows")]
fn find_prusa_slicer() -> anyhow::Result<PathBuf> {
    let app_path = std::path::PathBuf::from("C:\\Program Files\\PrusaSlicer\\PrusaSlicer.exe");
    if app_path.exists() {
        Ok(app_path)
    } else {
        anyhow::bail!("PrusaSlicer not found")
    }
}

// Find the prusaslicer executable path on Linux.
#[cfg(target_os = "linux")]
fn find_prusa_slicer() -> anyhow::Result<PathBuf> {
    let app_path = std::path::PathBuf::from("/usr/bin/prusa-slicer");
    if app_path.exists() {
        Ok(app_path)
    } else {
        anyhow::bail!("PrusaSlicer not found")
    }
}
