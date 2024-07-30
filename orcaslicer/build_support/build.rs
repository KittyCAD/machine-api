use std::{env, path};

use anyhow::Result;

/// We default to the rust target build dir so that the actions are cached and the
/// CI is a bunch faster.
const BUILD_DIR: &str = "target";

#[derive(Debug, Clone)]
pub struct Build {
    pub build_dir: String,
    pub target: crate::build_support::target::Target,
}

impl Build {
    /// Create a new build.
    pub fn new() -> Result<Self> {
        let mut current_dir = env::current_dir()?;
        let _ = current_dir.pop();
        let build_dir = current_dir.join(BUILD_DIR);
        println!("cargo:rerun-if-changed=../cpp/");

        let target = crate::build_support::target::Target::new();

        Ok(Build {
            target,
            build_dir: build_dir.to_str().unwrap().to_string(),
        })
    }

    /// Get the source directory for the given dependency by name.
    pub fn dep_source_dir(&self, name: &str) -> String {
        let build_dir = path::Path::new(&self.build_dir);
        build_dir
            .join("_deps")
            .join(name)
            .to_str()
            .expect("failed to convert path to string")
            .to_string()
    }

    /// Get the build directory for the given dependency by name.
    pub fn dep_build_dir(&self, name: &str) -> String {
        let source_dir = path::Path::new(&self.dep_source_dir(name)).join("build");
        let build_dir = source_dir.to_str().expect("failed to convert path to string");
        build_dir.to_string()
    }

    /// Get the install directory for the given dependency by name.
    pub fn dep_install_dir(&self, name: &str) -> String {
        let build_dir = path::Path::new(&self.dep_build_dir(name)).join("install");
        let install_dir = build_dir.to_str().expect("failed to convert path to string");
        install_dir.to_string()
    }

    /// Get the lib directory for the given dependency by name.
    pub fn dep_lib_dir(&self, name: &str) -> String {
        let install_dir = path::Path::new(&self.dep_install_dir(name)).join("lib");
        let lib_dir = install_dir.to_str().expect("failed to convert path to string");
        lib_dir.to_string()
    }

    /// Get the include directory for the given dependency by name.
    pub fn dep_include_dir(&self, name: &str) -> String {
        let install_dir = path::Path::new(&self.dep_install_dir(name)).join("include");
        let include_dir = install_dir.to_str().expect("failed to convert path to string");
        include_dir.to_string()
    }
}
