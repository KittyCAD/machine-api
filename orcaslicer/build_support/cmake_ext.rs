use std::env;

use anyhow::{bail, Result};

/// Set shared cmake args on commands.
pub trait CmakeExt {
    fn set_cmake_env(&mut self, build: &crate::build_support::build::Build) -> Result<&mut Self>;
}

impl CmakeExt for std::process::Command {
    fn set_cmake_env(&mut self, build: &crate::build_support::build::Build) -> Result<&mut Self> {
        if build.target.is_windows() {
            // If windows, configure compile.
            if build.target.abi == Some("gnu".to_owned()) {
                bail!("MinGW is not supported");
            }

            match build.target.architecture.as_str() {
                "x86_64" => self.args(["-A", "x64"]),
                "i686" => self.args(["-A", "Win32"]),
                _ => bail!("Unsupported architecture"),
            };
        } else if build.target.is_macos() {
            // If not windows, use clang.
            self.env("CC", env::var("CC").unwrap_or_else(|_| "clang".to_string()))
                .env("CXX", env::var("CXX").unwrap_or_else(|_| "clang++".to_string()))
                .env("ASM", env::var("ASM").unwrap_or_else(|_| "clang".to_string()))
                .env(
                    "CXXFLAGS",
                    env::var("CXXFLAGS").unwrap_or(format!("-target {}", build.target)),
                )
                .env(
                    "CFLAGS",
                    env::var("CFLAGS").unwrap_or(format!("-target {}", build.target)),
                );
        }

        Ok(self)
    }
}
