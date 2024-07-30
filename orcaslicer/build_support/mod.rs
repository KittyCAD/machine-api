pub mod build;
pub mod cmake_ext;
pub mod dependency;
pub mod target;

use std::{io::ErrorKind, process::Command};

use anyhow::{bail, Result};

pub fn run_command(cmd: &mut Command, program: &str) -> Result<()> {
    println!(
        "current_dir: {:?}\nrunning: {:?}",
        cmd.get_current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
        cmd
    );

    let output = match cmd.stdout(std::process::Stdio::piped()).output() {
        Ok(status) => status,
        Err(ref e) if e.kind() == ErrorKind::NotFound => {
            bail!(
                "{}",
                &format!("failed to execute command: {e}\nis `{program}` not installed?")
            );
        }
        Err(e) => bail!("{}", &format!("failed to execute command: {e:?}")),
    };
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let args = cmd.get_args().collect::<Vec<&std::ffi::OsStr>>();
        let mut args_str = String::new();
        for arg in args {
            args_str = format!("{} {}", args_str, arg.to_string_lossy())
        }
        bail!(
            "{}\n\n{}\n\n{}",
            stdout,
            stderr,
            &format!(
                "command `{}` did not execute successfully, got: {}",
                args_str, output.status
            )
        );
    }

    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn static_lib_filename(lib_name: &str) -> String {
    format!("lib{lib_name}.a")
}

#[cfg(target_os = "windows")]
pub fn static_lib_filename(lib_name: &str) -> String {
    format!("{}.lib", lib_name)
}

pub fn get_source_files() -> Vec<String> {
    // Add in our test files for when running tests.
    // We also _must_ add every bridged rs file to this list for them to be generated and linked correctly for C++
    vec![
        "src/engine.rs".to_string(),
        "src/stream.rs".to_string(),
        "src/brep.rs".to_string(),
        "src/export.rs".to_string(),
        "src/mesh.rs".to_string(),
        "src/solid.rs".to_string(),
        "src/tests.rs".to_string(),
        "src/demo.rs".to_string(),
    ]
}

#[cfg(target_os = "linux")]
fn build_nv_video_encoder() -> bool {
    true
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn build_nv_video_encoder() -> bool {
    false
}
