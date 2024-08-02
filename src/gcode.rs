use anyhow::Context;
use std::path::{Path, PathBuf};

use tokio::process::Command;

// Use Arc for shared ownership.
#[derive(Clone)]
pub struct GcodeFile {
    pub path: PathBuf,
}

impl GcodeFile {
    pub async fn from_stl_path(slicer: Slicer, slicer_config_path: &Path, stl_path: &Path) -> anyhow::Result<Self> {
        // Get the absolute path of the config file.
        let abs_path = slicer_config_path
            .canonicalize()
            .context("Failed to get the absolute path of the STL file")?;

        let gcode_path = match slicer {
            Slicer::Prusa => slice_stl_with_prusa_slicer(&abs_path, stl_path).await,
            Slicer::Orca => slice_stl_with_orca_slicer(&abs_path, stl_path).await,
        }?;

        Ok(Self { path: gcode_path })
    }
}

#[derive(Clone, Copy)]
pub enum Slicer {
    #[allow(dead_code)]
    Prusa,
    Orca,
}

async fn slice_stl_with_prusa_slicer(config_path: &Path, stl_path: &Path) -> anyhow::Result<PathBuf> {
    let gcode_path = stl_path.with_extension("gcode");

    let args: Vec<String> = vec![
        "--load".to_string(),
        config_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid slicer config path"))?
            .to_string(),
        "--support-material".to_string(),
        "--export-gcode".to_string(),
        stl_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid STL path"))?
            .to_string(),
        "--output".to_string(),
        gcode_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid G-code path"))?
            .to_string(),
    ];

    let output = Command::new("prusa-slicer")
        .args(&args)
        .output()
        .await
        .context("Failed to execute prusa-slicer command")?;

    println!("STDOUT: {}", std::str::from_utf8(&output.stdout)?);
    println!("STDERR: {}", std::str::from_utf8(&output.stderr)?);

    Ok(gcode_path.to_path_buf())
}

async fn slice_stl_with_orca_slicer(config_dir: &Path, stl_path: &Path) -> anyhow::Result<PathBuf> {
    let uid = uuid::Uuid::new_v4();
    let gcode_path = std::env::temp_dir().join(format!("{}.3mf", uid));
    let process_config = config_dir
        .join("process.json")
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid slicer config path"))?
        .to_string();
    let machine_config = config_dir
        .join("machine.json")
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid slicer config path"))?
        .to_string();
    let filament_config = config_dir
        .join("filament.json")
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid slicer config path"))?
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
            .ok_or_else(|| anyhow::anyhow!("Invalid G-code path"))?
            .to_string(),
        stl_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid STL path"))?
            .to_string(),
    ];

    let output = Command::new("/Applications/OrcaSlicer.app/Contents/MacOS/OrcaSlicer")
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

    Ok(gcode_path.to_path_buf())
}
