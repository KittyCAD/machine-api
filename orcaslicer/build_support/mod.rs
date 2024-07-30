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
