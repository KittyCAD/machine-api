use std::{
    path::{Path, PathBuf},
    process::Command,
};

// TODO don't clone this, maybe use Arc instead.
#[derive(Clone)]
pub struct GcodeSequence {
    pub lines: Vec<String>,
}

impl GcodeSequence {
    pub fn from_file_path(path: &Path) -> anyhow::Result<Self> {
        Ok(Self {
            lines: std::fs::read_to_string(path)?
                .lines() // split the string into an iterator of string slices
                .map(|s| {
                    let s = String::from(s);
                    match s.split_once(';') {
                        Some((command, _)) => command.trim().to_string(),
                        None => s.trim().to_string(),
                    }
                })
                .filter(|s| !s.is_empty()) // make each slice into a string
                .collect(),
        })
    }

    pub fn from_stl_path(slicer_config_path: &Path, stl_path: &Path) -> anyhow::Result<Self> {
        let gcode_path = slice_stl_with_prusa_slicer(slicer_config_path, stl_path)?;
        Self::from_file_path(&gcode_path)
    }
}

fn slice_stl_with_prusa_slicer(config_path: &Path, stl_path: &Path) -> anyhow::Result<PathBuf> {
    let gcode_path = stl_path.with_extension("gcode");
    let args: Vec<&str> = vec![
        "--load",
        config_path.to_str().ok_or(anyhow::anyhow!("bad slicer config path"))?,
        "--support-material",
        "--export-gcode",
        stl_path.to_str().ok_or(anyhow::anyhow!("bad stl path"))?,
        "--output",
        gcode_path.to_str().ok_or(anyhow::anyhow!("bad gcode path"))?,
    ];
    let _output = Command::new("prusa-slicer").args(args).output()?;

    Ok(gcode_path.to_path_buf())
}
