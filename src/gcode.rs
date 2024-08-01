use anyhow::Context;
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

// Use Arc for shared ownership.
#[derive(Clone)]
pub struct GcodeSequence {
    pub lines: Arc<Vec<String>>,
}

impl GcodeSequence {
    pub fn from_file_path(path: &Path) -> anyhow::Result<Self> {
        let lines = std::fs::read_to_string(path)?
            .lines() // split the string into an iterator of string slices
            .map(|s| {
                let s = String::from(s);
                match s.split_once(';') {
                    Some((command, _)) => command.trim().to_string(),
                    None => s.trim().to_string(),
                }
            })
            .filter(|s| !s.is_empty()) // make each slice into a string
            .collect();
        Ok(Self { lines: Arc::new(lines) })
    }

    pub fn from_stl_path(slicer: Slicer, slicer_config_path: &Path, stl_path: &Path) -> anyhow::Result<Self> {
        let gcode_path = match slicer {
            Slicer::Prusa => slice_stl_with_prusa_slicer(slicer_config_path, stl_path),
            Slicer::Orca => slice_stl_with_orca_slicer(slicer_config_path, stl_path),
        }?;
        Self::from_file_path(&gcode_path)
    }
}

#[derive(Clone, Copy)]
pub enum Slicer {
    Prusa,
    Orca,
}

fn slice_stl_with_prusa_slicer(config_path: &Path, stl_path: &Path) -> anyhow::Result<PathBuf> {
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
        .context("Failed to execute prusa-slicer command")?;

    println!("STDOUT: {}", std::str::from_utf8(&output.stdout)?);
    println!("STDERR: {}", std::str::from_utf8(&output.stderr)?);

    Ok(gcode_path.to_path_buf())
}

fn slice_stl_with_orca_slicer(config_dir: &Path, stl_path: &Path) -> anyhow::Result<PathBuf> {
    let gcode_path = stl_path.with_extension("3mf");
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

    let output = Command::new("orca-slicer")
        .args(&args)
        .output()
        .context("Failed to execute prusa-slicer command")?;

    println!("STDOUT: {}", std::str::from_utf8(&output.stdout)?);
    println!("STDERR: {}", std::str::from_utf8(&output.stderr)?);

    Ok(gcode_path.to_path_buf())
}
